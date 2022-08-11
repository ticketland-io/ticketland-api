use std::sync::Arc;
use actix_web::{web, HttpResponse};
use api_helpers::{
  services::data::{QueryString, exec_basic_db_read_endpoint},
};
use common_data::{
  repositories::event::read_events,
};
use crate::{
  utils::store::Store,
};

pub async fn exec(
  store: web::Data<Store>,
  qs: web::Query<QueryString>,
) -> HttpResponse {
  let skip = qs.skip.unwrap_or(0);
  let limit = qs.limit.unwrap_or(100);

  
  exec_basic_db_read_endpoint(
    Arc::clone(&store.neo4j),
    Box::new(qs.into_inner()),
    Box::new(move || {
      read_events(skip, limit)
    })
  ).await
}
