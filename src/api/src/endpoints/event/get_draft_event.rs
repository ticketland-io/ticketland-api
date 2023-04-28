use eyre::Result;
use actix_web::{
  web::{Data, Path},
  HttpResponse,
};
use ticketland_core::error::Error;
use api_helpers::{
  services::http::create_read_response,
  middleware::auth::AuthData
};
use crate::utils::store::Store;
use super::common::EventParams;

pub async fn exec(
  store: Data<Store>,
  auth: AuthData,
  params: Path<EventParams>,
) -> Result<HttpResponse, Error> {
  let mut postgres = store.pg_pool.connection().await?;
  let result = postgres.read_account_draft_event(
    auth.user.local_id.clone(),
    params.event_id.clone(),
  ).await;

  Ok(create_read_response(result, 0, 1))
}
