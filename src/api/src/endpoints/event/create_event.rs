use std::sync::Arc;
use serde::{Deserialize};
use actix_web::{web, HttpResponse};
use actix_multipart::Multipart;
use ticketland_core::error::Error;
use api_helpers::{
  middleware::auth::AuthData,
};
use crate::{
  utils::store::Store,
  services::metadata::{store_event},
};
use super::common::EventParams;

#[derive(Deserialize)]
pub struct Body {
  event_capacity: String,
}


pub async fn exec(
  store: web::Data<Store>,
  auth: AuthData,
  params: web::Path<EventParams>,
  payload: Multipart,
  body: web::Json<Body>,
) -> Result<HttpResponse, Error> {
  let event_id = params.event_id.clone();

  let metadata = store_event(
    Arc::clone(&store),
    event_id,
    body.event_capacity.clone(),
    auth.user.local_id,
    payload,
  ).await?;

  Ok(HttpResponse::Ok().json(metadata))
}
