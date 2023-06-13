use std::sync::Arc;
use actix_web::{
  web::{Data},
  HttpResponse,
};
use ticketland_core::error::Error;
use api_helpers::middleware::auth::AuthData;
use crate::{
  utils::store::Store,
  services::stripe::{
    Response,
    refresh_link,
  },
};

pub async fn exec(
  store: Data<Store>,
  auth: AuthData,
) -> Result<HttpResponse, Error> {
  refresh_link(Arc::clone(&store), auth.user.local_id)
  .await
  .map(|link| HttpResponse::Ok().json(Response {link: Some(link)}))
  .map_err(|err| err.into())
}
