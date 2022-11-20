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
    pub priceRange: Option<[u32; 2]>,
    pub date: Option<NaiveDateTime>,
    pub name: Option<String>,
  }
}

// #[derive(Serialize)]
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
  // TODO: add the correct default prop
  let category = qs.category.unwrap_or(0); 
  let priceRange = qs.priceRange.unwrap_or([0,2500]);
  let date = qs.date.unwrap_or(NaiveDateTime::parse_from_str("2020-04-12 22:10:57", "%Y-%m-%d %H:%M:%S").unwrap());
  let name = qs.name.clone().unwrap_or("".to_string());
  let mut postgres = store.postgres.lock().unwrap();
  let result = postgres.read_filtered_events(category, priceRange, date, name, skip, limit).await;

  create_read_response(result, skip, limit)
}
