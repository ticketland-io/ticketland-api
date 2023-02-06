use serde::{Deserialize};
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

QueryString! {
  pub struct QueryString {}
}

pub async fn exec(
  store: Data<Store>,
  qs: Query<QueryString>,
  auth: AuthData,
) -> Result<HttpResponse, Error> {
  let skip = qs.skip.unwrap_or(0);
  let limit = qs.limit.unwrap_or(100);
  let mut postgres = store.pg_pool.connection().await?;
  let result = postgres.read_account_events(
    auth.user.local_id.clone(),
    skip,
    limit,
  ).await;

  Ok(create_read_response(result, 0, 1))
}
