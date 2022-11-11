use std::sync::Arc;
use serde::{Deserialize};
use actix_web::{
  web::{Data, Path, Json},
  HttpResponse,
};
use api_helpers::{
  services::http::create_write_response,
  middleware::auth::AuthData,
};
use crate::{
  utils::store::Store,
};
use super::common::ListingParams;

#[derive(Deserialize)]
pub struct Body {
  ticket_nft: String,
}

pub async fn exec(
  store: Data<Store>,
  auth: AuthData,
  body: Json<Body>,
  params: Path<ListingParams>,
) -> HttpResponse {
  let mut postgres = store.postgres.lock().unwrap();
  let result = postgres.fill_sell_listing(
    params.listing_account.clone(),
    body.ticket_nft.clone(),
    auth.user.local_id.clone()
  ).await;


  create_write_response(result)
}
