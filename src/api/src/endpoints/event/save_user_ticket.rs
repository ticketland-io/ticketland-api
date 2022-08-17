use std::sync::Arc;
use serde::{Deserialize};
use actix_web::{
  web::{Data, Path, Json},
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
  repositories::event::{create_user_ticket},
};
use crate::{
  utils::store::Store,
};
use super::common::EventParams;

#[derive(Deserialize)]
pub struct Body {
  ticket_nft: String,
  event_index: u32,
  event_name: String,
}


pub async fn exec(
  store: Data<Store>,
  auth: AuthData,
  body: Json<Body>,
  params: Path<EventParams>,
) -> HttpResponse {
  exec_basic_db_write_endpoint(
    Arc::clone(&store.neo4j),
    Box::new(move || {
      create_user_ticket(
        auth.user.local_id.clone(),
        params.event_id.clone(),
        body.ticket_nft.clone(),
        body.event_index,
        body.event_name.clone(),
        Utc::now().timestamp(),
      )
    })
  ).await;

  HttpResponse::Created().finish()
}
