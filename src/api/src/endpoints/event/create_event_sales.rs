use serde::Deserialize;
use actix_web::{web, HttpResponse};
use ticketland_data::models::{
  sale::Sale,
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
  sales: Vec<Sale>,
  seat_ranges: Vec<SeatRange>,
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
