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
  middleware::auth::AuthData,
  services::{
    http::internal_server_error,
  },
};
use crate::{
  utils::store::Store,
  services::stripe::{
    Response,
    create_link,
  },
};

#[derive(Deserialize)]
pub struct Params {
  pub uid: String,
}

pub async fn exec(
  store: Data<Store>,
  auth: AuthData,
  params: Path<Params>,
) -> HttpResponse {
  create_link(Arc::clone(&store), auth.user.local_id.clone())
  .await
  .map(|link| HttpResponse::Ok().json(Response {link}))
  .unwrap_or_else(|error: Error| internal_server_error(Some(error)))
}
