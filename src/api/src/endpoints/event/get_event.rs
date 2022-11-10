use std::sync::Arc;
use actix_web::{web, HttpResponse};
use api_helpers::{middleware::auth::AuthData, services::data::exec_basic_db_read_endpoint_no_qs};
use common_data::{
  repositories::event::read_event_with_sales,
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

  exec_basic_db_read_endpoint_no_qs(
    Arc::clone(&store.neo4j),
    Box::new(move || {
      read_event_with_sales(event_id)
    })
  ).await
}
