use std::sync::Arc;
use serde::{Deserialize};
use actix_web::{web, HttpResponse};
use actix_multipart::Multipart;
use api_helpers::{
  middleware::auth::AuthData,
  services::http::internal_server_error,
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
) -> HttpResponse {
  let event_id = params.event_id.clone();

  store_event(
    Arc::clone(&store),
    event_id,
    body.event_capacity.clone(),
    auth.user.local_id,
    payload,
  ).await
  .map(|metadata| HttpResponse::Ok().json(metadata))
  .unwrap_or_else(|error| internal_server_error(Some(error.root_cause())))
}
