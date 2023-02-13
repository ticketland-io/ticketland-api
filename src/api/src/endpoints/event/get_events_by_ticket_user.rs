use serde::{Deserialize};
use eyre::{ContextCompat};
use actix_web::{
  web::{Data, Query},
  HttpResponse
};
use api_helpers::{
  QueryString,
  middleware::auth::AuthData,
  services::{
    http::create_read_response,
    data::QueryStringTrait,
  }
};
use ticketland_core::error::Error;
use crate::{
  utils::store::Store,
};
use chrono::NaiveDateTime;

QueryString! {
  pub struct QueryString {
    pub start_date_from: Option<i64>,
    pub start_date_to: Option<i64>,
  }
}
pub async fn exec(
  store: Data<Store>,
  qs: Query<QueryString>,
  auth: AuthData,
) -> Result<HttpResponse, Error> {
  let skip = qs.skip.unwrap_or(0);
  let limit = qs.limit.unwrap_or(100);
  let start_date_from = if let Some(date) = qs.start_date_from {
    Some(NaiveDateTime::from_timestamp_opt(date, 0).context("invalid start_date_from")?)
  } else { None };
  let start_date_to = if let Some(date) = qs.start_date_to {
    Some(NaiveDateTime::from_timestamp_opt(date, 0).context("invalid start_date_to")?)
  } else { None };
  let mut postgres = store.pg_pool.connection().await?;
  let result = postgres.read_account_ticket_events(
    auth.user.local_id.clone(),
    start_date_from,
    start_date_to,
    skip,
    limit,
  ).await;

  Ok(create_read_response(result, 0, 1))
}
