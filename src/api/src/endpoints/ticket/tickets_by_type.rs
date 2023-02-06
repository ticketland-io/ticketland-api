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
    pub event_id: Option<String>,
  }
}

pub async fn exec(
  store: Data<Store>,
  _auth: AuthData,
  qs: Query<QueryString>,
) -> Result<HttpResponse, Error> {
  let event_id = qs.event_id.clone().unwrap_or("".to_owned());
  let mut postgres = store.pg_pool.connection().await?;
  let result = postgres.read_attended_tickets(event_id).await;

  Ok(create_read_response(result, 0, 0))
}
