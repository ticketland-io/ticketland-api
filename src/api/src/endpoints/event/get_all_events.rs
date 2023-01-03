use serde::Serialize;
use actix_web::{web, HttpResponse};
use eyre::Result;
use ticketland_core::error::Error;
use api_helpers::{
  services::{
    data::QueryString,
    http::create_read_response,
  }
};
use crate::utils::store::Store;

#[derive(Serialize)]
pub struct BaseResponse<T: Serialize> {
  pub count: usize,
  pub skip: Option<u32>,
  pub limit: Option<u32>,
  pub result: T,
}


pub async fn exec(
  store: web::Data<Store>,
  qs: web::Query<QueryString>,
) -> Result<HttpResponse, Error> {
  let skip = qs.skip.unwrap_or(0);
  let limit = qs.limit.unwrap_or(100);

  let mut postgres = store.pg_pool.connection().await?;
  let result = postgres.read_events(skip, limit).await;

  Ok(create_read_response(result, skip, limit))
}
