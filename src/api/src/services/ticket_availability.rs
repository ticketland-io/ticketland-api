use std::{sync::Arc, collections::HashMap};
use eyre::{eyre, Result, ContextCompat};
use rand::Rng;
use sui_sdk::{SuiClient, rpc_types::{SuiObjectDataOptions, SuiData}};
use sui_types::{
  base_types::ObjectID,
  dynamic_field::{Field, DynamicFieldInfo},
};
use ticketland_core::services::redis;
use ticketland_data::connection_pool::ConnectionPool;
use ticketland_data::models::seat_range::SeatRange;
use ticketland_core::error::Error;
use ticketland_event_handler::services::ticket_purchase::pending_ticket_key;
use crate::map_err;
use super::bitmap;

async fn get_pending_tickets(redis_pool: &redis::ConnectionPool, event_id: &String) -> Result<Vec<u32>> {
  let mut redis = redis_pool.connection().await?;

  let pending_tickets_keys = redis
  .keys(&pending_ticket_key(event_id, "*"))
  .await?;

  let pending_tickets = pending_tickets_keys
  .iter()
  .map(|x| x
    .split(":")
    .last()
    //TODO: handle this unwrap better
    .unwrap_or("asd")
    .parse::<u32>()
  )
  .collect::<Result<Vec<u32>,_>>()?;

  Ok(pending_tickets)
}

fn create_available_seats_vec(
  seats: &Vec<u8>,
  seat_range: &SeatRange,
  pending_tickets: Vec<u32>,
) -> Vec<u32> {
  let SeatRange {l, r, ..} = *seat_range;
  let pending_seats: HashMap<u32, bool> = pending_tickets.into_iter().map(|s| (s, true)).collect();

  (l as u32..(r + 1) as u32)
  .filter_map(|i| {
    if !bitmap::is_set(i, &seats) && !pending_seats.contains_key(&i) {
      Some(i)
    } else {
      None
    }
  })
  .collect::<Vec<u32>>()
}

fn get_random_num(l: usize, r: usize) -> usize {
  rand::thread_rng().gen_range(l..r)
}

fn pick_random_seat(seats_bitmap: Vec<u32>) -> u32 {
  let random_index = get_random_num(0, seats_bitmap.len());

  seats_bitmap[random_index]
}

async fn fetch_dynamic_fields(rpc_client: Arc<SuiClient>, object_id: ObjectID) -> Result<Vec<DynamicFieldInfo>> {
  let mut fields = vec![];
  let mut cursor = None;

  loop {
    let response = rpc_client
      .read_api()
      .get_dynamic_fields(
        object_id,
        None,
        None,
      )
      .await?;

    fields.extend(response.data);

    if !response.has_next_page {break}
    cursor = response.next_cursor;
  }

  Ok(fields)
}

pub async fn get_seats(
  rpc_client: Arc<SuiClient>,
  object_id: ObjectID,
  total_seats: i32,
) -> Result<Vec<u8>> {
  let dynamic_fields = fetch_dynamic_fields(
    Arc::clone(&rpc_client),
    object_id
  ).await?;

  let object_ids = dynamic_fields
  .iter()
  .map(|data| data.object_id)
  .collect::<Vec<ObjectID>>();

  let seats_responses = rpc_client
  .read_api()
  .multi_get_object_with_options(
    object_ids,
    SuiObjectDataOptions::new().with_bcs(),
  )
  .await?;

  let seats_objects = seats_responses
  .into_iter()
  .map(|resp| map_err!(resp
    .object()?
    .bcs
    .as_ref()
    .context("Could not get ref")?
    .try_as_move()
    .context("Could not convert to MoveObject")?
    .deserialize())
  )
  .collect::<Result<Vec<Field<u64, u8>>, _>>()?;

  let mut seats: Vec<u8> = vec![0; total_seats as usize];

  seats_objects.iter().for_each(|seat_obj| {
    seats.insert(seat_obj.name as usize, seat_obj.value)
  });

  Ok(seats)
}

pub async fn get_available_seats(
  pg_pool: &ConnectionPool,
  redis_pool: &redis::ConnectionPool,
  rpc_client: Arc<SuiClient>,
  event_id: String,
  ticket_type_index: u8,
) -> Result<Vec<u32>> {
  let mut postgres = pg_pool.connection().await?;
  let event_result = postgres
  .read_event_with_ticket_types(event_id.clone(), false)
  .await?;
  let event = event_result.get(0).context("Event not found")?;
  let event_capacity_bitmap_address = event.event_capacity_bitmap_address.as_ref().context("Event capacity bitmap address not set")?;
  let n_tickets = event
  .ticket_types
  .iter()
  .fold(0, |acc, ticket_type| acc + ticket_type.n_tickets);

  let ticket_type = event.ticket_types
  .get(ticket_type_index as usize)
  .context("Ticket type not in event")?;

  let pending_tickets = get_pending_tickets(&redis_pool, &event_id.clone()).await?;
  let seats = get_seats(
    Arc::clone(&rpc_client),
    ObjectID::from_hex_literal(&event_capacity_bitmap_address)?,
    n_tickets
    ).await?;

  // TODO: atm we assume that each sale has a single seat_range. However, the db schema allows for multiple
  // So for the time being we will use the first and only seat_range stored in the db
  let available_seats = create_available_seats_vec(&seats, &ticket_type.seat_range, pending_tickets);

  if available_seats.is_empty() {
    return Err(Error::GenericError("No seat available".to_string()).into());
  }

  Ok(available_seats)
}

pub async fn get_next_seat_index(
  pg_pool: &ConnectionPool,
  redis_pool: &redis::ConnectionPool,
  rpc_client: Arc<SuiClient>,
  event_id: String,
  ticket_type_index: u8,
) -> Result<u32> {
  let available_seats: Vec<u32> = get_available_seats(
    pg_pool,
    redis_pool,
    rpc_client,
    event_id,
    ticket_type_index
  ).await?;

  let random_seat = pick_random_seat(available_seats);

  Ok(random_seat)
}

pub async fn is_seat_available(
  pg_pool: &ConnectionPool,
  redis_pool: &redis::ConnectionPool,
  rpc_client: Arc<SuiClient>,
  event_id: String,
  ticket_type_index: u8,
  seat_index: u32,
) -> Result<bool> {
  let available_seats: Vec<u32> = get_available_seats(
    pg_pool,
    redis_pool,
    rpc_client,
    event_id,
    ticket_type_index
  ).await?;

  Ok(available_seats
  .iter()
  .position(|seat| *seat == seat_index)
  .is_some())
}
