use actix_web::{
  web::Data,
  HttpResponse,
};
use eyre::Result;
use ticketland_core::error::Error;
use api_helpers::{middleware::auth::AuthData, services::http::internal_server_error};
use crate::{
  utils::store::Store,
};

pub async fn exec(
  store: Data<Store>,
  auth: AuthData,
) -> Result<HttpResponse, Error> {
  let mut postgres = store.pg_pool.connection().await?;

  Ok(
    postgres
    .read_stripe_account(auth.user.local_id)
    .await
    .map(|result| HttpResponse::Ok().json(result))
    .unwrap_or_else(|error| internal_server_error(Some(error.root_cause())))
  )
}
