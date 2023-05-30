use actix_web::{
  web::Data,
  HttpResponse,
};
use eyre::Result;
use ticketland_core::error::Error;
use api_helpers::{
  middleware::auth::AuthData, 
  services::http::create_response
};
use crate::{
  utils::store::Store,
};

pub async fn exec(
  store: Data<Store>,
  auth: AuthData,
) -> Result<HttpResponse, Error> {
  let mut postgres = store.pg_pool.connection().await?;
  let result =postgres.read_stripe_account(auth.user.local_id).await;

  Ok(create_response(result))
}
