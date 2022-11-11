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
use ticketland_data::models::buy_listing::BuyListing;
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
  let result = postgres.create_buy_listing().await;

  exec_basic_db_write_endpoint(
    Arc::clone(&store.neo4j),
    Box::new(move || {
      create_buy_listing(
        auth.user.local_id.clone(),
        body.event_id.clone(),
        params.listing_account.clone(),
        body.bid_price.clone(),
      )
    })
  ).await;

  HttpResponse::Created().finish()
}
