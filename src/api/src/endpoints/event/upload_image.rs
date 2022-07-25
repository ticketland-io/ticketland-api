use actix_web::{web, HttpResponse};
use actix_multipart::Multipart;
use ticketland_core::error::Error;
use api_helpers::{
  middleware::auth::AuthData,
};
use crate::{
  utils::store::Store,
  services::metadata::upload_image_to_s3,
};
use super::common::EventParams;

pub async fn exec(
  store: web::Data<Store>,
  payload: Multipart,
  _auth: AuthData,
  params: web::Path<EventParams>,
) -> Result<HttpResponse, Error> {
  let event_id = params.event_id.clone();
	
  upload_image_to_s3(
    &store.minio,
    &event_id,
    payload,
  ).await?;

  Ok(HttpResponse::Ok().finish())
}
