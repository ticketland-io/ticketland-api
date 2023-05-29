use std::sync::Arc;
use eyre::Result;
use actix_web::{web, HttpResponse};
use actix_multipart::Multipart;
use ticketland_core::error::Error;
use api_helpers::{
  middleware::auth::AuthData, 
  services::http::create_write_response,
};
use crate::{
  utils::store::Store,
  services::event::{store_event},
};
use super::common::EventParams;

pub async fn exec(
  store: web::Data<Store>,
  auth: AuthData,
  params: web::Path<EventParams>,
  payload: Multipart
) -> Result<HttpResponse, Error> {
  let event_id = params.event_id.clone();

  let metadata = store_event(
    Arc::clone(&store),
    event_id,
    auth.user.local_id,
    payload,
  ).await;

  Ok(create_write_response(metadata))
}
