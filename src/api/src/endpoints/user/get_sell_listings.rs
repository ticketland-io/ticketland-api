use serde::{Deserialize};
use actix_web::{
  web::{Data, Query, Path},
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
    pub event_id: Option<String>,
  }
}

#[derive(Deserialize)]
pub struct Params {
  pub uid: String,
}

pub async fn exec(
  store: Data<Store>,
  params: Path<Params>,
  qs: Query<QueryString>,
) -> Result<HttpResponse, Error> {
  let skip = qs.skip.unwrap_or(0);
  let limit = qs.limit.unwrap_or(100);
  let mut postgres = store.pg_pool.connection().await?;
  let result = postgres.read_sell_listings_for_account(
    qs.event_id.clone(),
    params.uid.clone(),
    skip,
    limit
  ).await;

  create_read_response(result, skip, limit)
}
