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
  // TODO: make sure this event id belongs to the current user. Or even better use a custom authz middleware
  let event_id = params.event_id.clone();

  let ipfs_read_stream = S3Stream::new(
    format!("{}-event_file.png", event_id),
    1024,
    Arc::clone(&store.minio),
  );

  Ok(
    HttpResponse::Ok()
    .insert_header(ContentDisposition {
      disposition: DispositionType::Attachment,
      parameters: vec![
        DispositionParam::Filename(format!("{}-event_file.png", event_id)),
      ],
    })
    .streaming(ipfs_read_stream)
  )
}
