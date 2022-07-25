use std::sync::Arc;
use actix_web::{web, HttpResponse};
use actix_multipart::Multipart;
use ticketland_core::error::Error;
use api_helpers::{
  middleware::auth::AuthData,
};
use crate::{
  utils::store::Store,
  services::metadata::store_event,
};
use super::common::EventParams;

pub async fn exec(
  store: web::Data<Store>,
  params: web::Path<EventParams>,
  payload: Multipart,
  auth: AuthData,
) -> Result<HttpResponse, Error> {
  let event_id = params.event_id.clone();

  store_event(
    Arc::clone(&store),
    event_id,
    auth.user.local_id,
    payload,
  ).await?;

  Ok(HttpResponse::Ok().finish())
}
