use std::sync::{Arc, RwLock};
use actix_web::{
  web,
  HttpResponse,
};
use tokio::io::duplex;
use ticketland_core::{
  error::Error,
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
	let (async_writer, async_reader) = duplex(10 * 1024 * 1024);

  Arc::clone(&store.minio).get_object_stream(
    &format!("{}-event_image.png", event_id),
    Arc::new(RwLock::new(async_writer)),
  ).await
  .map_err(Into::<Error>::into)?;

  let result = store.ipfs.upload_stream(async_reader)
  .await
  .map_err(|error| {
    println!("{:?}", error);
    error
  })?;

  println!("{:?}", result);

  Ok(HttpResponse::Ok().finish())
}
