use std::{sync::Arc, collections::HashMap};
use eyre::Result;
use rand::Rng;
use program_artifacts::{
  ticket_sale::account_data::EventCapacity, event_registry::account_data::EventId,
};
use solana_web3_rust::utils::pubkey_from_str;
use common_data::{
  helpers::{send_read},
  models::event::Event,
  repositories::{
    event::{read_event},
    sale::read_event_sale,
  },
  models::sale::{Sale, SeatRange},
};
use ticketland_core::error::Error;
use ticketland_event_handler::services::ticket_purchase::pending_ticket_key;
use program_artifacts::{ticket_sale::pda as ticket_sale_pda};
use ticketland_utils::bitmap;
use crate::utils::store::Store;

async fn get_pending_tickets(store: Arc<Store>, event_id: &String) -> Result<Vec<u32>> {
  let mut redis = store.redis.lock().unwrap();

  let pending_tickets = redis
  .get_mult(&pending_ticket_key(event_id, "*"))
  .await
  .map(|values| {
    values
    .iter()
    .map(|x| x.parse::<u32>().unwrap())
    .collect::<Vec<u32>>()
  })?;

  Ok(pending_tickets)
}

fn create_seats_bitmap(
  seats: Vec<u8>,
  seat_range: SeatRange,
  pending_tickets: Vec<u32>,
) -> Vec<u32> {
  let SeatRange {l, r} = seat_range;
  let pending_seats: HashMap<u32, bool> = pending_tickets.into_iter().map(|s| (s, true)).collect();

  (l..r)
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

pub async fn get_next_seat_index(
  store: Arc<Store>,
  event_id: &EventId,
  ticket_type_index: u8,
) -> Result<u32> {
  let (query, db_query_params) = read_event(event_id.db_val());
  let event = send_read(Arc::clone(&store.neo4j), query, db_query_params)
  .await
  .map(TryInto::<Event>::try_into)?
  .unwrap();

  let event_capacity_state = pubkey_from_str(&event.event_capacity).unwrap();
  let event_capacity_data = store
  .rpc_client
  .get_anchor_account_data::<EventCapacity>(&event_capacity_state)
  .await?;

  let sale = ticket_sale_pda::sale(
    &store.config.ticket_sale_program_state,
    ticket_type_index,
    &event_id.val(),
  )
  .0;

  let (query, db_query_params) = read_event_sale(sale.to_string().clone());
  let sale = send_read(Arc::clone(&store.neo4j), query, db_query_params)
  .await
  .map(TryInto::<Sale>::try_into)?
  .unwrap();

  let pending_tickets = get_pending_tickets(Arc::clone(&store), &event_id.db_val()).await?;

  let seats_bitmap = create_seats_bitmap(event_capacity_data.seats, sale.seat_range, pending_tickets);

  if seats_bitmap.is_empty() {
    return Err(Error::GenericError("No seat available".to_string()).into());
  }

  let random_seat = pick_random_seat(seats_bitmap);

  Ok(random_seat)
}
