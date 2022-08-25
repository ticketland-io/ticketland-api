use std::sync::Arc;
use serde::{Deserialize};
use actix_web::{
  web::{Data, Json},
  HttpResponse,
};
use chrono::{Utc};
use api_helpers::{
  services::{
    data::{exec_basic_db_write_endpoint},
  },
  middleware::auth::AuthData,
};
use common_data::{
  repositories::ticket::{create_user_ticket},
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
) -> HttpResponse {
  exec_basic_db_write_endpoint(
    Arc::clone(&store.neo4j),
    Box::new(move || {
      create_user_ticket(
        auth.user.local_id.clone(),
        body.event_id.clone(),
        body.ticket_nft.clone(),
        body.ticket_metadata.clone(),
        body.seat_index,
        body.seat_name.clone(),
        Utc::now().timestamp(),
      )
    })
  ).await;

  HttpResponse::Created().finish()
}
