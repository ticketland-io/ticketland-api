use serde::{Deserialize};
use actix_web::{
  web::{Data, Path, Json},
  HttpResponse,
};
use eyre::Result;
use ticketland_core::error::Error;
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
) -> Result<HttpResponse, Error> {
  let mut postgres = store.pg_pool.connection().await?;
  let result = postgres.fill_buy_listing(
    params.listing_account.clone(),
    body.ticket_nft.clone(),
    auth.user.local_id.clone()
  ).await;

  create_write_response(result)
}
