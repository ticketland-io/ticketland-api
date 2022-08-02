use std::sync::Arc;
use actix_web::{
  web::{Data, Path},
  HttpResponse,
};
use ticketland_core::{
  error::Error,
  streams::s3_stream::S3Stream,
};
use futures::future;
use futures_util::StreamExt;
use api_helpers::{
  services::http::internal_server_error,
  middleware::auth::AuthData,
};
use common_data::{
  helpers::{send_read},
  models::event::Event,
  repositories::event::{read_event},
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
  let (query, db_query_params) = read_event(event_id);

  let event = send_read(Arc::clone(&store.neo4j), query, db_query_params)
  .await
  .map(|db_result| {
    TryInto::<Event>::try_into(db_result)
  })
  .unwrap_or_else(|error: Error| Err(error));

  if let Err(error) = event {
    return Ok(internal_server_error(Some(error)))
  }
  
  let event = event.unwrap();

  // TODO: This code will be moved to an external services. This endpoint will simply push 
  // a message to RabbitMQ indicating that the given event_id was created on the blockchain.
  let ipfs_read_stream = S3Stream::new(
    format!("{}-event_file.{}", event.event_id, event.file_type),
    1024,
    Arc::clone(&store.minio),
  );

  // Get all the data from the stream
  let mut data = vec![];
  ipfs_read_stream
  .for_each(|val| {
    let mut slice = val.unwrap().into_iter().collect::<Vec<u8>>();
    data.append(&mut slice);
    future::ready(())
  })
  .await;

  let _ = store.ipfs.upload(data)
  .await
  .map_err(Into::<Error>::into)?;

  Ok(HttpResponse::Ok().finish())
}
