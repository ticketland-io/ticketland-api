use eyre::Result;
use ticketland_core::error::Error;
use actix_web::{
  web::{Data, Path},
  HttpResponse,
};
use api_helpers::{
  services::{
    http::create_read_response,
  },
  middleware::auth::AuthData,
};
use crate::{
  utils::store::Store,
};
use super::common::EventParams;

pub async fn exec(
  store: Data<Store>,
  _auth: AuthData,
  params: Path<EventParams>,
) -> Result<HttpResponse, Error> {
  let event_id = params.event_id.clone();
  let mut postgres = store.pg_pool.connection().await?;
  let result = postgres.read_attended_tickets(event_id).await;

  Ok(create_read_response(result, 0, 0))
}
