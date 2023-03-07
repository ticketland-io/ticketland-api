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
  let mut postgres = store.pg_pool.connection().await?;
  let event = postgres
  .read_event_with_sales(event_id)
  .await?
  .get(0)
  .context("event now found")?;

  let ticket_image_types = event
  .ticket_images
  .iter()
  .map(|t| t.ticket_image_type)
  .collect::<Vec<i16>>();

  store.new_event_queue
  .new_event(
    event.event_id.clone(),
    event.file_type.clone().context("file_type missing")?,
    ticket_image_types,
  )
  .await?;

  Ok(HttpResponse::Ok().finish())
}
