use std::sync::Arc;
use serde::{Deserialize};
use actix_web::{
  web::{Data, Json},
  HttpResponse,
};
use futures_util::TryFutureExt;
use chrono::{Duration};
use api_helpers::{
  middleware::auth::AuthData,
  services::http::internal_server_error,
};
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
  ticket_type_index: u8,
}

pub async fn exec(
  store: Data<Store>,
  auth: AuthData,
  body: Json<Body>,
) -> HttpResponse {
  let lock = store.redlock.lock(body.ticket_nft.as_bytes(), Duration::seconds(10).num_milliseconds() as usize).await;

  if let Err(error) = lock {
    return internal_server_error(Some(error.root_cause()))
  }

  // store the record in Redis so this ticket is considered unavailable
  let mut redis = store.redis.lock().unwrap();
  let redis_key = pending_ticket_key(&body.event_id, &body.ticket_nft);
  let store = Arc::clone(&store);
  let store_copy = Arc::clone(&store);

  redis.set_ex(&redis_key, &"1", Duration::minutes(5).num_milliseconds() as usize)
  .and_then(|_| {
    let store = Arc::clone(&store);

    async move {
      let (query, db_query_params) = upsert_user_ticket(
        auth.user.local_id.clone(),
        body.event_id.clone(),
        body.ticket_nft.clone(),
        body.ticket_metadata.clone(),
        body.seat_index,
        body.seat_name.clone(),
        body.ticket_type_index,
      );

      send_write(
        Arc::clone(&store.neo4j),
        query,
        db_query_params,
      ).await
      .map_err(Into::<_>::into)
    }
  })
  .and_then(|_| {
    async move { 
      store_copy.redlock.unlock(lock.unwrap()).await;
      Ok(())
    }
  })
  .await
  .map(|_| HttpResponse::Created().finish())
  .unwrap_or_else(|error| internal_server_error(Some(error.root_cause())))
}
