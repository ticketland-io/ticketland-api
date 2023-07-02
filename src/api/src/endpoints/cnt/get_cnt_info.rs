use serde::{Deserialize};
use actix_web::{
  web::{Data, Path},
  HttpResponse,
};
use ticketland_core::error::Error;
use crate::{
  utils::store::Store,
};
use api_helpers::{
  services::http::create_read_response,
  middleware::auth::AuthData,
};

#[derive(Deserialize)]
pub struct Params {
  pub cnt_sui_address: String,
}

pub async fn exec(
  store: Data<Store>,
  _auth: AuthData,
  params: Path<Params>,
) -> Result<HttpResponse, Error> {
  let mut postgres = store.pg_pool.connection().await?;
  let result = postgres.read_cnt(params.cnt_sui_address.to_owned()).await;

  create_read_response(result, 0, 1)
}
