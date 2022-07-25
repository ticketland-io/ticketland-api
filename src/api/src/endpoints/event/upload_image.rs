use std::sync::Arc;
use actix_web::{web, HttpResponse};
use actix_multipart::Multipart;
use ticketland_core::error::Error;
use api_helpers::{
  middleware::auth::AuthData,
};
use crate::{
  utils::store::Store,
  services::metadata::store_new_tmp_image,
};
use super::common::EventParams;

pub async fn exec(
  store: web::Data<Store>,
  payload: Multipart,
  auth: AuthData,
  params: web::Path<EventParams>,
) -> Result<HttpResponse, Error> {
  let event_id = params.event_id.clone();
	
  store_new_tmp_image(
    Arc::clone(&store),
    event_id,
    auth.user.local_id,
    payload,
  ).await?;

  Ok(HttpResponse::Ok().finish())
}
