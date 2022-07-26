use actix_web::{
  web,
  HttpResponse,
};
use tokio::io::duplex;
use tokio_util::io::ReaderStream;
use ticketland_core::{
  error::Error,
  services::pinata::Pinata,
};
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
  let event_id = params.event_id.clone();

  // TODO: This code will be moved to an external services. This endpoint will simply push 
  // a message to RabbitMQ indicating that the given event_id was created on the blockchain.
	let (mut async_writer, async_reader) = duplex(10 * 1024 * 1024);
  let stream_reader = ReaderStream::new(async_reader);
  
  store.minio.get_object_stream(
    &format!("{}-event_image.png", event_id),
    &mut async_writer,
  ).await
  .map_err(Into::<Error>::into)?;

  let pinata = Pinata::new(
    store.config.pinata_api_uri.clone(),
    store.config.pinata_api_token.clone(),
  );

  pinata.upload(&format!("{}-event_image.png", event_id), stream_reader)
  .await
  .map_err(|error| {
    println!("{:?}", error);
    error
  })?;

  Ok(HttpResponse::Ok().finish())
}
