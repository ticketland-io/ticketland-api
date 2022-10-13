use actix_web::{
  web::{Data},
  HttpResponse,
};
use ticketland_core::{
  error::Error,
};
use api_helpers::{
  middleware::auth::AuthData,
  services::{
    http::internal_server_error,
  },
};
use crate::{
  utils::store::Store,
  services::stripe::create_account_link,
};

pub async fn exec(
  store: Data<Store>,
  _auth: AuthData,
) -> HttpResponse {
  create_account_link(
    store.config.stripe_key.clone()
  )
  .await
  .map(|link| {
    HttpResponse::Found()
    .append_header(("Location", link.url))
    .finish()
  })
  .unwrap_or_else(|error: Error| internal_server_error(Some(error)))
}
