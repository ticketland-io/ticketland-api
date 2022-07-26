use actix_web::{
  web,
  HttpResponse,
  http::header::{ContentDisposition, DispositionType, DispositionParam},
};
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
	let (mut async_writer, async_reader) = tokio::io::duplex(10 * 1024 * 1024);
  let stream_reader = tokio_util::io::ReaderStream::new(async_reader);
  
  store.minio.get_object_stream(
    &format!("{}-event_image.png", event_id),
    &mut async_writer,
  ).await
  .map_err(|error| {
    println!("{:?}", error);
    error
  })?;

  Ok(
    HttpResponse::Ok()
    .insert_header(ContentDisposition {
      disposition: DispositionType::Attachment,
      parameters: vec![
        DispositionParam::Filename(format!("{}-event_image.png", event_id)),
      ],
    })
    .streaming(stream_reader)
  )
}
