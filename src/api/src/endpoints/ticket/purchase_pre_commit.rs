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
use crate::{
  utils::store::Store,
  services::ticket_purchase::pending_ticket_key,
};

#[derive(Deserialize)]
pub struct Body {
  event_id: String,
  ticket_nft: String,
}

pub async fn exec(
  store: Data<Store>,
  _auth: AuthData,
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

  redis.set_ex(&redis_key, &"1", Duration::minutes(1).num_milliseconds() as usize)
  .and_then(|()| {
    async move { 
      store.redlock.unlock(lock.unwrap()).await;
      Ok(())
    }
  })
  .await
  .map(|_| HttpResponse::Created().finish())
  .unwrap_or_else(|error| internal_server_error(Some(error.root_cause())))
}
