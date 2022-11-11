use serde::Deserialize;
use actix_web::{web, HttpResponse};
use ticketland_data::models::{
  sale::NewSale,
  seat_range::NewSeatRange,
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
  seat_ranges: Vec<NewSeatRange>,
}

pub async fn exec(
  store: web::Data<Store>,
  _auth: AuthData,
  params: web::Path<EventParams>,
  body: web::Json<Body>
) -> HttpResponse {
  let mut postgres = store.postgres.lock().unwrap();
  let result = postgres.upsert_sales(body.sales.clone(), body.seat_ranges.clone()).await;

  create_write_response(result)
}
