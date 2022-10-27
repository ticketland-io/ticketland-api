use std::sync::Arc;
use actix_web::{web, HttpResponse};
use common_data::{
  repositories::sale::upsert_event_sale,
  models::sale::Sale,
};
use api_helpers::{
  middleware::auth::AuthData,
  services::{
    data::{exec_basic_db_write_endpoint},
  },
};
use crate::{
  utils::store::Store,
};
use super::common::EventParams;

pub async fn exec(
  store: web::Data<Store>,
  _auth: AuthData,
  params: web::Path<EventParams>,
  body: web::Json<Vec<Sale>>
) -> HttpResponse {
  let event_id = params.event_id.clone();

  exec_basic_db_write_endpoint(
    Arc::clone(&store.neo4j),
    Box::new(move || {
      upsert_event_sale(event_id, body.0)
    })
  ).await;

  HttpResponse::Created().finish()
}
