use std::sync::Arc;
use serde::{Deserialize};
use actix_web::{
  web::{Data, Json},
  HttpResponse,
};
use api_helpers::{
  middleware::auth::AuthData,
  services::{
    http::internal_server_error,
  },
};
use crate::{
  utils::store::Store,
  services::stripe::{
    CheckoutSessionResponse,
    create_primary_sale_checkout,
  },
};

#[derive(Deserialize)]
pub struct Body {
  event_id: String,
  ticket_nft: String,
  sale_account: String,
  recipient: String,
  seat_index: u32,
  seat_name: String,
}

pub async fn exec(
  store: Data<Store>,
  auth: AuthData,
  body: Json<Body>,
) -> HttpResponse {
  create_primary_sale_checkout(
    Arc::clone(&store),
    auth.user.local_id,
    body.sale_account.clone(),
    body.event_id.clone(),
    body.ticket_nft.clone(),
    body.recipient.clone(),
    body.seat_index,
    body.seat_name.clone(),
  )
  .await
  .map(|session_id| HttpResponse::Ok().json(CheckoutSessionResponse {session_id}))
  .unwrap_or_else(|error| internal_server_error(Some(error.root_cause())))
}
