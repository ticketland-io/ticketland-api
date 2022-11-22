use actix_web::{
  web::{Data, Path},
  HttpResponse,
};
use eyre::{Result, ContextCompat};
use ticketland_core::error::Error;
use api_helpers::{
  middleware::auth::AuthData,
};
use crate::utils::store::Store;
use super::common::EventParams;

pub async fn exec(
  store: Data<Store>,
  _auth: AuthData,
  params: Path<EventParams>,
) -> Result<HttpResponse, Error> {
  let event_id = params.event_id.clone();
  let mut postgres = store.postgres.lock().await;
  let event = postgres.read_event(event_id).await?;
  
  store.new_event_queue
  .new_event(event.event_id, event.file_type.context("file_type missing")?)
  .await?;

  Ok(HttpResponse::Ok().finish())
}
