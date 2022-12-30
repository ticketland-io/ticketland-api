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
use ticketland_data::models::buy_listing::NewBuyListing;
use crate::{
  utils::store::Store,
};
use super::common::ListingParams;

#[derive(Deserialize)]
pub struct Body {
  bid_price: i64,
  event_id: String,
  n_listing: i64
}

pub async fn exec(
  store: Data<Store>,
  auth: AuthData,
  body: Json<Body>,
  params: Path<ListingParams>,
) -> Result<HttpResponse, Error> {
  let mut postgres = store.pg_pool.connection().await?;
  let result = postgres.upsert_buy_listing(NewBuyListing {
    account_id: &auth.user.local_id,
    event_id: &body.event_id,
    sol_account: &params.listing_account,
    bid_price: body.bid_price,
    n_listing: body.n_listing,
    is_open: true,
    draft: false,
  }).await;

  Ok(create_write_response(result))
}
