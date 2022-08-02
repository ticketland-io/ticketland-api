use std::sync::Arc;
use actix_web::{
  web,
  HttpResponse,
  http::header::{ContentDisposition, DispositionType, DispositionParam},
};

use ticketland_core::{
  error::Error,
  streams::s3_stream::S3Stream,
};
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
  services::utils::get_event_file_path,
};
use super::common::EventParams;

pub async fn exec(
  store: web::Data<Store>,
  _auth: AuthData,
  params: web::Path<EventParams>,
) -> Result<HttpResponse, Error> {
  // TODO: make sure this event id belongs to the current user. Or even better use a custom authz middleware
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
  
  let file_path = get_event_file_path(&event.event_id, &event.file_type);

  let ipfs_read_stream = S3Stream::new(
    file_path.clone(),
    1024,
    Arc::clone(&store.minio),
  );

  Ok(
    HttpResponse::Ok()
    .insert_header(ContentDisposition {
      disposition: DispositionType::Attachment,
      parameters: vec![
        DispositionParam::Filename(file_path.clone()),
      ],
    })
    .streaming(ipfs_read_stream)
  )
}
