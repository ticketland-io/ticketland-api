use std::sync::Arc;
use actix_web::{web, HttpResponse};
use api_helpers::{
  services::data::{QueryString, exec_basic_db_read_endpoint},
  middleware::auth::AuthData,
};
use common_data::{
  repositories::design::read_ticket_designs,
};
use crate::{
  utils::store::Store,
};

pub async fn exec(
  store: web::Data<Store>,
  qs: web::Query<QueryString>,
  auth: AuthData,
) -> HttpResponse {
  
  exec_basic_db_read_endpoint(
    Arc::clone(&store.neo4j),
    Box::new(qs.into_inner()),
    Box::new(move || {
      read_ticket_designs(auth.user.local_id.clone())
    })
  ).await
}
