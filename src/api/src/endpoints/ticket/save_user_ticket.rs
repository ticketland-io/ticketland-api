use actix_web::{
  web::{Data, Json},
  HttpResponse,
};
use api_helpers::{
  middleware::auth::AuthData,
};
use ticketland_core::error::Error;
use ticketland_data::{
  models::ticket::Ticket,
};
use ticketland_event_handler::{
  services::ticket_purchase::pending_ticket_key,
};
use crate::{
  utils::store::Store,
};

pub async fn exec(
  store: Data<Store>,
  _auth: AuthData,
  body: Json<Ticket>,
) -> Result<HttpResponse, Error> {
  // 1. Update DB
  let mut postgres = store.postgres.lock().unwrap();
  postgres.upsert_user_ticket(body.0.clone()).await?;

  // 2. Remove ending key from Redis
  let mut redis = store.redis.lock().unwrap();
  let redis_key = pending_ticket_key(&body.event_id, &body.ticket_nft);
  
  redis.delete(&redis_key).await?;

  Ok(HttpResponse::Created().finish())
}
