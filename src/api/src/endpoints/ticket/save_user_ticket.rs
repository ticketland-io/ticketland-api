use std::sync::Arc;
use serde::{Deserialize};
use actix_web::{
  web::{Data, Json},
  HttpResponse,
};
use api_helpers::{
  middleware::auth::AuthData,
};
use ticketland_core::error::Error;
use ticketland_data::{
  modles::ticket::Ticket,
};
use ticketland_event_handler::{
  services::ticket_purchase::pending_ticket_key,
};
use crate::{
  utils::store::Store,
};

#[derive(Deserialize)]
pub struct Body {
  event_id: String,
  ticket_nft: String,
  ticket_metadata: String,
  seat_index: u32,
  seat_name: String,
  ticket_type_index: u8,
}

pub async fn exec(
  store: Data<Store>,
  auth: AuthData,
  body: Json<Ticket>,
) -> Result<HttpResponse, Error> {
  // 1. Update DB
  let mut postgres = store.postgres.lock().unwrap();
  let result = postgres.upsert_user_ticket(body.0).await;

  // 2. Remove ending key from Redis
  let mut redis = store.redis.lock().unwrap();
  let redis_key = pending_ticket_key(&body.event_id, &body.ticket_nft);
  
  redis.delete(&redis_key).await?;

  Ok(HttpResponse::Created().finish())
}
