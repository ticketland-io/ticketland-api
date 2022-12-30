use actix_web::{web, HttpResponse};
use api_helpers::{
  middleware::auth::AuthData,
  services::http::create_read_response,
};
use ticketland_core::error::Error;
use crate::{
  utils::store::Store,
};

pub async fn exec(
  store: web::Data<Store>,
  auth: AuthData,
) -> Result<HttpResponse, Error> {
  // TODO: add pagination functionality
  let mut postgres = store.pg_pool.connection().await?;
  let result = postgres.read_account_events(auth.user.local_id.clone()).await;

  Ok(create_read_response(result, 0, 1))
}
