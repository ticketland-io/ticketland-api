use std::{sync::Arc, str::FromStr};
use serde::{Serialize, Deserialize};
use chrono::{Utc, Duration};
use eyre::{Result, ContextCompat};
use actix_web::{
  web::{Data, Json},
  HttpResponse,
};
use stripe::{Client, PaymentIntent, PaymentIntentId};
use ticketland_core::error::Error;
use api_helpers::{middleware::auth::AuthData, services::http::internal_server_error};
use ticketland_event_handler::services::ticket_purchase::pending_ticket_key;
use crate::{utils::store::Store, services::ticket_availability::is_seat_available};

#[derive(Serialize)]
pub struct Response {
  pub seat_index: u32,
  pub seat_name: String,
}

#[derive(Deserialize)]
pub struct Body {
  event_id: String,
  payment_intent_id: String,
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

pub async fn exec(
  store: Data<Store>,
  auth: AuthData,
  body: Json<Body>,
) -> Result<HttpResponse, Error> {
  let payment_intent = retrieve_intent(store.config.stripe_key.clone(), body.payment_intent_id.clone()).await?;

  let seat_index = payment_intent.metadata.get("seat_index").context("missing seat_index")?.parse::<u32>().unwrap();
  let seat_name = payment_intent.metadata.get("seat_name").context("missing seat_name")?.to_string();
  let buyer_uid = payment_intent.metadata.get("buyer_uid").context("missing buyer_uid")?.to_string();
  let ticket_type_index = payment_intent.metadata.get("ticket_type_index").context("missing ticket_type_index")?.parse::<u8>().unwrap();

  // 1. Check that the request sender is the same as the user that initialized the payment
  if buyer_uid != auth.user.local_id {
    return internal_server_error(Some(Error::GenericError("User not payment initialized".to_owned())))
  }

  // 2. Check if the payment intent has expired; We can load the payment intent from the db and check manually if N mins has passed
  if let Err(err) = check_payment_intent(&payment_intent).await {
    return internal_server_error(Some(err))
  }

  // TODO: this is maybe not valid, as the fiat-checkout-manager will have
  // already set the pending_ticket_key which means this ticket will be considered
  // as not available

  // 3. Check if the seat_index key is in Redis; If so then the ticket is not available
  // let mut redis = store.redis_pool.connection().await?;
  // let redis_key = pending_ticket_key(&body.event_id.clone(), &seat_index.to_string());
  // if let Ok(_) = redis.get(&redis_key).await {
  //   return internal_server_error(Some(Error::GenericError("Ticket not available".to_owned())))
  // }

  // Check if the seat is available
  // if !is_seat_available(
  //   &store.pg_pool,
  //   &store.redis_pool,
  //   Arc::clone(&store.rpc_client),
  //   body.event_id.clone(),
  //   ticket_type_index,
  //   seat_index,
  // ).await? {
  //   return internal_server_error(Some(Error::GenericError("Ticket not available".to_owned())))
  // }

  Ok(HttpResponse::Ok().json(Response {
    seat_index,
    seat_name,
  }))
}
