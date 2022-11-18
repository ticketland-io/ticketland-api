use actix_web::{web, HttpResponse};
use api_helpers::{
  middleware::auth::AuthData,
  services::http::create_read_response,
};
use crate::{
  utils::store::Store,
};
use super::common::EventParams;


pub async fn exec(
  store: web::Data<Store>,
  _auth: AuthData,
  params: web::Path<EventParams>,
) -> HttpResponse {
  let event_id = params.event_id.clone();
  let mut postgres = store.postgres.lock().await;
  let result = postgres.read_event_with_sales(event_id).await;

  create_read_response(result, 0, 1)
}
