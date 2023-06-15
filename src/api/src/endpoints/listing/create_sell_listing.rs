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
use ticketland_data::models::sell_listing::NewSellListing;
use crate::{
  utils::store::Store,
};
use super::common::ListingParams;

#[derive(Deserialize)]
pub struct Body {
  event_id: String,
  ticket_nft: String,
  ask_price: i64,
}

pub async fn exec(
  store: Data<Store>,
  auth: AuthData,
  body: Json<Body>,
  params: Path<ListingParams>,
) -> Result<HttpResponse, Error> {
  let mut postgres = store.pg_pool.connection().await?;
  let result = postgres.upsert_sell_listing(NewSellListing {
    account_id: &auth.user.local_id,
    cnt_nft: &body.ticket_nft,
    event_id: &body.event_id,
    sol_account: &params.listing_account,
    ask_price: body.ask_price,
    is_open: true,
    draft: false,
  }).await;

  create_write_response(result)
}
