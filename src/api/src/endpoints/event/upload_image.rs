use actix_web::{web, HttpResponse};
use actix_multipart::Multipart;
use ticketland_core::error::Error;
use api_helpers::{
  middleware::auth::AuthData,
};
use futures_util::stream::StreamExt;
use crate::{
  utils::store::Store,
};
use super::common::EventParams;

pub async fn exec(
  store: web::Data<Store>,
  mut payload: Multipart,
  _auth: AuthData,
  params: web::Path<EventParams>,
) -> Result<HttpResponse, Error> {
  let event_id = params.event_id.clone();
	let mut content = vec![];

  while let Some(item) = payload.next().await {
    let mut field = item
    .map_err(|_| Error::S3Error)?;


    // Field in turn is stream of *Bytes* object
    while let Some(chunk) = field.next().await {
      let chunk = chunk.map_err(|_| Error::S3Error)?;
      content.push(chunk);
    }
  }

	let content =  content.concat();
  store.minio.upload(
    &format!("{}-event_image", event_id),
    content.as_ref()
  )
  .await
  .map_err(|_| Error::S3Error)?;

  Ok(HttpResponse::Ok().finish())
}
