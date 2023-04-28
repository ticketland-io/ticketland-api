use serde::Deserialize;
use eyre::Result;
use actix_web::{
  web::{Data, Query},
  HttpResponse,
};
use ticketland_core::error::Error;
use api_helpers::{
  QueryString,
  services::{
    http::create_read_response,
    data::QueryStringTrait,
  },
  middleware::auth::AuthData
};
use crate::utils::store::Store;

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
  auth: AuthData,
  qs: Query<QueryString>,
) -> Result<HttpResponse, Error> {
  let skip = qs.skip.unwrap_or(0);
  let limit = qs.limit.unwrap_or(100);

  let mut postgres = store.pg_pool.connection().await?;
  let result = postgres.read_account_draft_events(
    auth.user.local_id.clone(),
    skip,
    limit,
  ).await;

  Ok(create_read_response(result, skip, limit))
}
