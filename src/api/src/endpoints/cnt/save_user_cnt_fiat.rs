use chrono::Duration;
use serde::{Deserialize};
use actix_web::{
  web::{Data, Json},
  HttpResponse,
};
use api_helpers::{
  middleware::auth::AuthData, 
  services::http::create_write_response,
};
use ticketland_core::error::Error;
use ticketland_data::models::cnt::CNT;
use ticketland_event_handler::{
  services::ticket_purchase::pending_ticket_key,
};
use crate::{
  utils::store::Store,
};

#[derive(Deserialize)]
pub struct Body {
  event_id: String,
  seat_index: u32,
  seat_name: String,
  ticket_type_index: u8,
  txb_bytes: String,
  signature: String,
}

pub async fn exec(
  store: Data<Store>,
  auth: AuthData,
  body: Json<Body>,
) -> Result<HttpResponse, Error> {
  let mut postgres = store.pg_pool.connection().await?;
  let account = postgres.read_account_by_id(auth.user.local_id.clone()).await?;

  // 1. Update DB
  let cnt = CNT {
    cnt_sui_address: None,
    event_id: body.event_id.clone(),
    account_id: auth.user.local_id.clone(),
    created_at: None,
    ticket_type_index: body.ticket_type_index as i16,
    seat_name: body.seat_name.clone(),
    seat_index: body.seat_index as i32,
    attended: false,
    draft: true
  };

  postgres.upsert_user_cnt(cnt).await?;

  // 2. store the record in Redis so this ticket is considered unavailable
  let mut redis = store.redis_pool.connection().await?;
  let redis_key = pending_ticket_key(&body.event_id, &body.seat_index.to_string());

  redis.set_ex(
    &redis_key,
    "1",
    Duration::days(1).num_milliseconds() as usize,
  ).await?;

  // TODO: check signed_tx is correct

  // 3. Send new ticket purchase message to rabbitmq to execute operator purchase tx
  let result = store.ticket_purchase_queue.new_ticket_purchase(
    auth.user.local_id.clone(),
    body.event_id.clone(),
    account.pubkey,
    body.seat_index.to_string(),
    body.seat_name.clone(),
    body.txb_bytes.clone(),
    body.signature.clone(),
  ).await;

  create_write_response(result)
}
