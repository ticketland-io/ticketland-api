use std::sync::Arc;
use serde::{Deserialize};
use actix_web::{
  web::{Data, Path},
  HttpResponse,
};
use ticketland_core::{
  error::Error,
};
use api_helpers::{
  services::{
    http::internal_server_error,
  },
};
use crate::{
  utils::store::Store,
  services::stripe::{
    Response,
    refresh_link,
  },
};

#[derive(Deserialize)]
pub struct Params {
  pub uid: String,
}

pub async fn exec(
  store: Data<Store>,
  params: Path<Params>,
) -> HttpResponse {
  refresh_link(Arc::clone(&store), params.uid.clone())
  .await
  .map(|link| HttpResponse::Ok().json(Response {link}))
  .unwrap_or_else(|error: Error| internal_server_error(Some(error)))
}
