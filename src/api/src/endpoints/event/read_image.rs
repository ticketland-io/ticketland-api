use actix_web::{web, HttpResponse};
use ticketland_core::error::Error;
use api_helpers::{
  middleware::auth::AuthData,
};
use crate::{
  utils::store::Store,
};
use super::common::EventParams;

pub async fn exec(
  store: web::Data<Store>,
  _auth: AuthData,
  params: web::Path<EventParams>,
) -> Result<HttpResponse, Error> {
  // TODO: make sure this event id belongs to the current user
  let event_id = params.event_id.clone();
	let (mut asyncwriter, _) = tokio::io::duplex(256 * 1024);

  store.minio.get_object_stream(
    &format!("{}-event_image", event_id),
    &mut asyncwriter,
  ).await?;

  Ok(HttpResponse::Ok().streaming(tokio_util::io::ReaderStream::new(asyncwriter)))
}
