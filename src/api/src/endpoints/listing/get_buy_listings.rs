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
    data::QueryStringTrait,
    http::create_read_response,
  }
};
use crate::{
  utils::store::Store,
};

QueryString! {
  pub struct QueryString {
    pub event_id: String,
  }
}

pub async fn exec(
  store: Data<Store>,
  qs: Query<QueryString>,
) -> Result<HttpResponse, Error> {
  let skip = qs.skip.unwrap_or(0);
  let limit = qs.limit.unwrap_or(100);
  let mut postgres = store.pg_pool.connection().await?;
  let result = postgres.read_buy_listings_for_event(qs.event_id.clone(), skip, limit).await;

  Ok(create_read_response(result, skip, limit))
}
