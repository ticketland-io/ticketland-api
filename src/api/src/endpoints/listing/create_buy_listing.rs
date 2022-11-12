use serde::{Deserialize};
use actix_web::{
  web::{Data, Path, Json},
  HttpResponse,
};
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
}

pub async fn exec(
  store: Data<Store>,
  auth: AuthData,
  body: Json<Body>,
  params: Path<ListingParams>,
) -> HttpResponse {
  let mut postgres = store.postgres.lock().unwrap();
  let result = postgres.create_buy_listing(NewBuyListing {
    account_id: &auth.user.local_id,
    event_id: &body.event_id,
    sol_account: &params.listing_account,
    bid_price: body.bid_price,
    is_open: true,
  }).await;

  create_write_response(result)
}
