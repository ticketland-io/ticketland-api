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
    create_checkout_session,
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
  todo!()
}
