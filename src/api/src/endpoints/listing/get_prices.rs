use actix_web::{
  web::{Data, Query},
  HttpResponse,
};
use serde::{Deserialize};
use api_helpers::{
  QueryString,
  services::{
    http::create_read_response,
    data::QueryStringTrait,
  }
};
use crate::utils::store::Store;


QueryString! {
  pub struct QueryString {
    pub interval: u64,
    pub start_ts: u64,
  }
}

pub async fn exec(
  store: Data<Store>,
  qs: Query<QueryString>,
) -> HttpResponse {
  let mut postgres = store.postgres.lock().await;
  let result = postgres.read_average_listings_price(qs.interval, qs.start_ts).await;

  create_read_response(result, 0, 1)
}
