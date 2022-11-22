use std::sync::Arc;
use actix_web::{
  web,
  HttpResponse,
  http::header::{ContentDisposition, DispositionType, DispositionParam},
};
use eyre::{Result, ContextCompat};
use ticketland_core::{
  error::Error,
  streams::s3_stream::S3Stream,
};
use api_helpers::{
  middleware::auth::AuthData,
};
use ticketland_event_handler::{
  services::path,
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
  // TODO: make sure this event id belongs to the current user. Or even better use a custom authz middleware
  let mut postgres = store.postgres.lock().await;
  let event = postgres.read_event(params.event_id.clone()).await?;
  let file_path = path::get_event_file_path(&event.event_id, "ticket_image", &event.file_type.context("file_type missing")?);

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
