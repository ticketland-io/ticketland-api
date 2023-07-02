
use actix_web::{
  web::{Data, Path},
  HttpResponse,
};
use eyre::Result;
use ticketland_core::error::Error;
use api_helpers::{
  services::http::create_write_response,
  middleware::auth::AuthData,
};
use crate::{
  utils::store::Store,
};
use super::common::OfferParams;

pub async fn exec(
  store: Data<Store>,
  auth: AuthData,
  params: Path<OfferParams>,
) -> Result<HttpResponse, Error> {
  let mut postgres = store.pg_pool.connection().await?;
  let result = postgres.cancel_offer(
    auth.user.local_id.clone(),
    params.offer_id.clone()
  ).await;

  create_write_response(result)
}
