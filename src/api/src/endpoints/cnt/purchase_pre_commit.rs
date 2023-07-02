use std::sync::Arc;
use serde::{Serialize, Deserialize};
use eyre::Result;
use actix_web::{
  web::{Data, Json},
  HttpResponse,
};
use ticketland_core::error::Error;
use api_helpers::{middleware::auth::AuthData};
use crate::{
  utils::store::Store,
  services::{
    ticket_purchase_pre_commit::store_ticket_purchase_pre_commit,
    ticket_availability::get_next_seat_index,
  },
};

#[derive(Serialize)]
pub struct Response {
  pub seat_index: u32,
  pub seat_name: String,
}

#[derive(Deserialize)]
pub struct Body {
  event_id: String,
  seat_index: Option<u32>,
  seat_name: Option<String>,
  ticket_type_index: u8,
}

fn has_all_optional_fields(body: &Json<Body>) -> bool {
  body.seat_index.is_some()
  && body.seat_name.is_some()
}

pub async fn exec(
  store: Data<Store>,
  auth: AuthData,
  body: Json<Body>,
) -> Result<HttpResponse, Error> {
  let seat_index;
  let seat_name;

  if has_all_optional_fields(&body) {
    seat_index = body.seat_index.unwrap();
    seat_name = body.seat_name.as_ref().unwrap().clone();
  } else {
    seat_index = get_next_seat_index(
      &store.pg_pool,
      &store.redis_pool,
      Arc::clone(&store.rpc_client),
      body.event_id.clone(),
      body.ticket_type_index,
    )
    .await?;

    seat_name = seat_index.to_string();
  }

  store_ticket_purchase_pre_commit(
    Arc::clone(&store),
    auth.user.local_id.clone(),
    body.event_id.clone(),
    seat_index,
    seat_name.clone(),
    body.ticket_type_index
  )
  .await
  .map(|_| HttpResponse::Created().json(Response {
    seat_index,
    seat_name,
  }))
  .map_err(|err| err.into())
}
