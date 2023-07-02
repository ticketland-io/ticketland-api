use actix_web::{
  web::{Data, Json},
  HttpResponse,
};
use serde::Deserialize;
use api_helpers::{
  middleware::auth::AuthData,
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
  pub sui_address: String,
  pub event_id: String,
  pub ticket_type_index: i16,
  pub seat_name: String,
  pub seat_index: i32,
}

pub async fn exec(
  store: Data<Store>,
  auth: AuthData,
  body: Json<Body>,
) -> Result<HttpResponse, Error> {
  // 1. Update DB
  let mut postgres = store.pg_pool.connection().await?;

  let ticket = CNT {
    cnt_sui_address: Some(body.sui_address.clone()),
    event_id: body.event_id.clone(),
    account_id: auth.user.local_id.clone(),
    created_at: None,
    ticket_type_index: body.ticket_type_index,
    seat_name: body.seat_name.clone(),
    seat_index: body.seat_index,
    attended: false,
    draft: false
  };

  postgres.upsert_user_cnt(ticket).await?;
  
  // 2. Remove ending key from Redis
  let mut redis = store.redis_pool.connection().await?;
  let redis_key = pending_ticket_key(&body.event_id, &body.seat_index.to_string());
  
  redis.delete(&redis_key).await?;

  Ok(HttpResponse::Created().finish())
}
