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
use ticketland_data::models::listing::NewListing;
use crate::{
  utils::store::Store,
};
use super::common::ListingParams;

#[derive(Deserialize)]
pub struct Body {
  event_id: String,
  listing_sui_address: String,
  cnt_sui_address: String,
  ask_price: i64,
}

pub async fn exec(
  store: Data<Store>,
  auth: AuthData,
  body: Json<Body>,
  params: Path<ListingParams>,
) -> Result<HttpResponse, Error> {
  let mut postgres = store.pg_pool.connection().await?;
  let result = postgres.upsert_listing(NewListing {
    listing_id: &params.listing_id,
    account_id: &auth.user.local_id,
    cnt_sui_address: &body.cnt_sui_address,
    event_id: &body.event_id,
    listing_sui_address: Some(&body.listing_sui_address),
    ask_price: body.ask_price,
    is_open: true,
    draft: false,
  }).await;

  create_write_response(result)
}
