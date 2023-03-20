use actix_web::{web, HttpResponse};
use eyre::Result;
use ticketland_core::error::Error;
use api_helpers::services::http::create_read_response;
use crate::{
  utils::store::Store,
};
use super::common::EventParams;


pub async fn exec(
  store: web::Data<Store>,
  params: web::Path<EventParams>,
) -> Result<HttpResponse, Error> {
  let event_id = params.event_id.clone();
  let mut postgres = store.pg_pool.connection().await?;
  let result = postgres.read_event_with_sales(event_id, false).await;

  Ok(create_read_response(result, 0, 1))
}
