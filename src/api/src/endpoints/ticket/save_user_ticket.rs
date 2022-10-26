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
use common_data::{
  helpers::{send_write},
  repositories::ticket::{upsert_user_ticket},
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
}

pub async fn exec(
  store: Data<Store>,
  auth: AuthData,
  body: Json<Body>,
) -> Result<HttpResponse, Error> {
  // 1. Update DB
  let (query, db_query_params) = upsert_user_ticket(
    auth.user.local_id.clone(),
    body.event_id.clone(),
    body.ticket_nft.clone(),
    body.ticket_metadata.clone(),
    body.seat_index,
    body.seat_name.clone(),
  );

  send_write(
    Arc::clone(&store.neo4j),
    query,
    db_query_params,
  ).await?;

  // 2. Remove ending key from Redis
  let mut redis = store.redis.lock().unwrap();
  let redis_key = pending_ticket_key(&body.event_id, &body.ticket_nft);
  
  redis.delete(&redis_key).await?;

  Ok(HttpResponse::Created().finish())
}
