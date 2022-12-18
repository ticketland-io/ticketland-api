use serde::{Deserialize, Serialize};
use eyre::{Result, ContextCompat};
use actix_web::{
  web::{Data, Query},
  HttpResponse,
};
use chrono::NaiveDateTime;
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
    pub price_range: Option<[u32; 2]>,
    pub start_date: Option<i64>,
    pub end_date: Option<i64>,
    pub search: Option<String>,
  }
}

#[derive(Serialize)]
pub struct BaseResponse<T: Serialize> {
  pub count: usize,
  pub skip: Option<u32>,
  pub limit: Option<u32>,
  pub result: T,
}

pub async fn exec(
  store: Data<Store>,
  qs: Query<QueryString>,
) -> HttpResponse {
  let skip = qs.skip.unwrap_or(0);
  let limit = qs.limit.unwrap_or(100);
  let category = qs.category;
  let price_range = qs.price_range.unwrap_or([0,2000]);
  let start_date = qs.start_date.map(|date|{
    // TODO: remove unwrap
    return NaiveDateTime::from_timestamp_opt(date, 0).context("invalid start_date").unwrap();
  });
  let end_date = qs.end_date.map(|date|{
    // TODO: remove unwrap
    return NaiveDateTime::from_timestamp_opt(date, 0).context("invalid start_date").unwrap();
  });
  
  let name = qs.search.clone();

  let mut postgres = store.postgres.lock().await;
  let result = postgres.read_filtered_events(
    category,
    price_range,
    start_date,
    end_date,
    name,
    skip,
    limit
  ).await;

  create_read_response(result, skip, limit)
}
