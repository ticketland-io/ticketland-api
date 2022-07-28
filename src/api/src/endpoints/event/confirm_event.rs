use std::sync::Arc;
use actix_web::{
  web::{Data, Path},
  HttpResponse,
};
use ticketland_core::{
  error::Error,
  streams::s3_stream::S3Stream,
};
use futures_util::StreamExt;
use api_helpers::{
  middleware::auth::AuthData,
};
use crate::{
  utils::store::Store,
};
use super::common::EventParams;

pub async fn exec(
  store: Data<Store>,
  _auth: AuthData,
  params: Path<EventParams>,
) -> Result<HttpResponse, Error> {
  let event_id = params.event_id.clone();

  // TODO: This code will be moved to an external services. This endpoint will simply push 
  // a message to RabbitMQ indicating that the given event_id was created on the blockchain.
  let ipfs_read_stream = S3Stream::new(
    format!("{}-event_image.png", event_id),
    1024,
    Arc::clone(&store.minio),
  );

  // Get all the data from the stream
  let mut data = vec![];
  ipfs_read_stream
  .for_each(|val| {
    let mut slice = val.unwrap().into_iter().collect::<Vec<u8>>();
    data.append(&mut slice);
    futures::future::ready(())
  })
  .await;


  println!("{:?}", data);

  let result = store.ipfs.upload(data)
  .await
  .map_err(|error| {
    println!("{:?}", error);
    error
  })?;

  println!("{:?}", result);

  Ok(HttpResponse::Ok().finish())
}
