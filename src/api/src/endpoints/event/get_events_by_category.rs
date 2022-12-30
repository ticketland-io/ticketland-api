use serde::{Deserialize};
use actix_web::{
  web::{Data, Query},
  HttpResponse,
};
use eyre::Result;
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
    pub category: i16,
  }
}

pub async fn exec(
  store: Data<Store>,
  qs: Query<QueryString>,
) -> Result<HttpResponse, Error> {
  let skip = qs.skip.unwrap_or(0);
  let limit = qs.limit.unwrap_or(100);
  let mut postgres = store.pg_pool.connection().await?;
  let result = postgres.read_events_by_category(qs.category, skip, limit).await;

  Ok(create_read_response(result, skip, limit))
}
