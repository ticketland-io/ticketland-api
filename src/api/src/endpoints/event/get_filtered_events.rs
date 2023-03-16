use serde::Deserialize;
use eyre::{Result, ContextCompat};
use actix_web::{
  web::{Data, Query},
  HttpResponse,
};
use chrono::NaiveDateTime;
use ticketland_core::error::Error;
use api_helpers::{
  QueryString,
  services::{
    http::create_read_response,
    data::QueryStringTrait,
  }
};
use crate::{
  utils::store::Store,
};

QueryString! {
  pub struct QueryString {
    pub category: Option<i16>,
    pub price_range_l: Option<u32>,
    pub price_range_r: Option<u32>,
    pub start_date_from: Option<i64>,
    pub start_date_to: Option<i64>,
    pub search: Option<String>,
  }
}

pub async fn exec(
  store: Data<Store>,
  qs: Query<QueryString>,
) -> Result<HttpResponse, Error> {
  let skip = qs.skip.unwrap_or(0);
  let limit = qs.limit.unwrap_or(100);
  let category = qs.category;

  let price_range = if let (Some(price_range_l), Some(price_range_r)) = (qs.price_range_l, qs.price_range_r) {
    Some((price_range_l, price_range_r))
  } else { None };

  let start_date_from = if let Some(date) = qs.start_date_from {
    Some(NaiveDateTime::from_timestamp_opt(date, 0).context("invalid start_date_from")?)
  } else { None };
  let start_date_to = if let Some(date) = qs.start_date_to {
    Some(NaiveDateTime::from_timestamp_opt(date, 0).context("invalid start_date_to")?)
  } else { None };

  let name = qs.search.clone();

  let mut postgres = store.pg_pool.connection().await?;
  let result = postgres.read_filtered_events(
    category,
    price_range,
    start_date_from,
    start_date_to,
    name,
    skip,
    limit
  ).await;

  Ok(create_read_response(result, skip, limit))
}
