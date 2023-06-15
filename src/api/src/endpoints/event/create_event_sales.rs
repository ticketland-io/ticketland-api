use serde::Deserialize;
use actix_web::{web, HttpResponse};
use eyre::Result;
use ticketland_core::error::Error;
use ticketland_data::models::{
  sale::NewSale,
  seat_range::SeatRange,
};
use api_helpers::{
  middleware::auth::AuthData,
  services::http::create_write_response
};
use crate::{
  utils::store::Store,
};
use super::common::EventParams;

#[derive(Deserialize)]
pub struct Body {
  sales: Vec<NewSale>,
  seat_ranges: Vec<SeatRange>,
}

pub async fn exec(
  store: web::Data<Store>,
  _auth: AuthData,
  _params: web::Path<EventParams>,
  body: web::Json<Body>
) -> Result<HttpResponse, Error> {
  let mut postgres = store.pg_pool.connection().await?;
  let result = postgres.upsert_sales(body.sales.clone(), body.seat_ranges.clone()).await;

  create_write_response(result)
}
