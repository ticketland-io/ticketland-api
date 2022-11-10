use serde::Serialize;
use actix_web::{web, HttpResponse};
use eyre::Result;
use api_helpers::{
  services::data::{QueryString, exec_basic_db_read_endpoint},
};
use ticketland_core::error::Error;
use crate::{
  utils::store::Store,
};

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

  let mut postgres = store.postgres.lock().unwrap();
  let result = postgres.read_events(skip, limit).await?;

  Ok(
    HttpResponse::Ok()
    .json(BaseResponse {
      count: result.len(),
      result,
      skip: Some(skip),
      limit: Some(limit),
    })
  )
}
