use actix_web::{
  web::{Data, Query},
  HttpResponse,
};
use serde::{Deserialize};
use ticketland_core::error::Error;
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
    pub event_id: String,
    pub interval: u64,
    pub start_ts: u64,
  }
}

pub async fn exec(
  store: Data<Store>,
  qs: Query<QueryString>,
) -> Result<HttpResponse, Error> {
  let mut postgres = store.pg_pool.connection().await?;
  let result = postgres.read_average_sales_price(
    qs.event_id.clone(),
    qs.interval,
    qs.start_ts,
  ).await;

  create_read_response(result, 0, 1)
}
