use serde::{Deserialize};
use eyre::Result;
use ticketland_core::error::Error;
use actix_web::{
  web::{Data, Query},
  HttpResponse,
};
use api_helpers::{
  QueryString,
  services::{
    data::QueryStringTrait,
    http::create_read_response,
  },
  middleware::auth::AuthData,
};
use crate::{
  utils::store::Store,
};

QueryString! {
  pub struct QueryString {
    event_id: Option<String>,
  }
}

pub async fn exec(
  store: Data<Store>,
  auth: AuthData,
  qs: Query<QueryString>,
) -> Result<HttpResponse, Error> {
  // TODO: we want to return user events for all events if this is none
  let skip = qs.skip.unwrap_or(0);
  let limit = qs.limit.unwrap_or(100);
  let mut postgres = store.pg_pool.connection().await?;
  let result = postgres.read_user_tickets(
    auth.user.local_id.clone(),
    qs.event_id.clone(),
    skip,
    limit
  ).await;

  create_read_response(result, skip, limit)
}
