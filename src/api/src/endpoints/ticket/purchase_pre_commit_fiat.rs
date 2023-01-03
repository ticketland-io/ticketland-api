use std::{sync::Arc, str::FromStr};
use serde::{Serialize, Deserialize};
use chrono::{Utc, Duration};
use eyre::{Result, ContextCompat};
use actix_web::{
  web::{Data, Json},
  HttpResponse,
};
use solana_web3_rust::utils::pubkey_from_str;
use stripe::{Client, PaymentIntent, PaymentIntentId};

use ticketland_core::error::Error;
use api_helpers::{middleware::auth::AuthData, services::http::internal_server_error};
use program_artifacts::{
  event_registry::account_data::EventId,
  ticket_sale::{self, account_data::SeatReservation}
};
use ticketland_event_handler::services::ticket_purchase::pending_ticket_key;
use crate::utils::store::Store;

#[derive(Serialize)]
pub struct Response {
  pub seat_index: u32,
  pub ticket_nft: String,
  pub seat_name: String,
}

#[derive(Deserialize)]
pub struct Body {
  event_id: String,
  // ticket_nft: String,
  // sale_account: String,
  payment_intent_id: String,
  // seat_index: u32,
  // seat_name: String,
}

async fn retrieve_intent(
  stripe_key: String,
  payment_intent_id: String,
) -> Result<PaymentIntent> {
  let client = Client::new(stripe_key.clone());
  let payment_intent_id = PaymentIntentId::from_str(&payment_intent_id).unwrap();

  PaymentIntent::retrieve(&client, &payment_intent_id, &vec![])
  .await
  .map_err(Into::<_>::into)
}

async fn check_payment_intent(payment_intent: &PaymentIntent) -> Result<(), Error> {
  if Utc::now().timestamp() > (payment_intent.created + Duration::minutes(30).num_seconds()) {
    return Err(Error::GenericError("Payment expired".to_owned()))
  }

  Ok(())
}

async fn check_seat_reservation(
  store: Arc<Store>,
  sale_account: String,
  seat_index: u32,
  seat_name: String,
  uid: String,
) -> Result<(), Error> {
  let sale_account = pubkey_from_str(&sale_account.clone())?;
  let seat_reservation = ticket_sale::pda::seat_reservation(&sale_account, seat_index, &seat_name).0;
  let seat_reservation_data = store
  .rpc_client
  .get_anchor_account_data::<SeatReservation>(&seat_reservation)
  .await?;

  let mut postgres = store.pg_pool.connection().await?;
  let account = postgres.read_account_by_id(uid).await?;

  let latest_slot = store.rpc_client.get_slot().await?;

  if latest_slot > seat_reservation_data.valid_until {
    return Err(Error::GenericError("Reservation expired".to_owned()))
  }

  let account_pub_key = pubkey_from_str(&account.pubkey)?;
  if seat_reservation_data.recipient != account_pub_key {
    return Err(Error::GenericError("Wrong recipient".to_owned()))
  }

  Ok(())
}

pub async fn exec(
  store: Data<Store>,
  auth: AuthData,
  body: Json<Body>,
) -> Result<HttpResponse, Error> {
  let payment_intent = retrieve_intent(store.config.stripe_key.clone(), body.payment_intent_id.clone()).await?;

  let seat_index = payment_intent.metadata.get("seat_index").context("")?.parse::<u32>().unwrap();
  let seat_name = payment_intent.metadata.get("seat_name").context("")?.to_string();
  let ticket_nft = payment_intent.metadata.get("ticket_nft").context("")?.to_string();
  let sale_account = payment_intent.metadata.get("sale_account").context("")?.to_string();
  let buyer_uid = payment_intent.metadata.get("buyer_uid").context("")?.to_string();

  // 1. Check that the request sender is the same as the user that initialized the payment
  if buyer_uid != auth.user.local_id {
    return Ok(internal_server_error(Some(Error::GenericError("User not payment initialized".to_owned()))))
  }

  // 2. Check if the payment intent has expired; We can load the payment intent from the db and check manually if N mins has passed
  if let Err(err) = check_payment_intent(&payment_intent).await {
    return Ok(internal_server_error(Some(err)))
  }

  let event_id = EventId(body.event_id.clone());

  // 3. Check if the ticket_nft key is in Redis; If so then the ticket is not available
  let mut redis = store.redis_pool.connection().await?;
  let redis_key = pending_ticket_key(&event_id.db_val(), &ticket_nft);
  if let Ok(_) = redis.get(&redis_key).await {
    return Ok(internal_server_error(Some(Error::GenericError("Ticket not available".to_owned()))))
  }

  // 4. Check that the seat reservation has not expired
  if let Err(err) = check_seat_reservation(
    Arc::clone(&store),
    sale_account.clone(),
    seat_index,
    seat_name.clone(),
    auth.user.local_id.clone()
  ).await {
    return Ok(internal_server_error(Some(err)))
  }

  Ok(HttpResponse::Ok().json(Response {
    seat_index,
    seat_name,
    ticket_nft,
  }))

}
