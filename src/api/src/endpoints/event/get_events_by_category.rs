use std::sync::Arc;
use serde::{Deserialize};
use actix_web::{
  web::{Data, Query},
  HttpResponse,
};
use api_helpers::{
  QueryString,
  services::data::{QueryStringTrait, exec_basic_db_read_endpoint},
};
use ticketland_data::{
  repositories::event::read_events_by_category
};
use crate::{
  utils::store::Store,
};

QueryString! {
  pub struct QueryString {
    pub category: String,
  }
}

pub async fn exec(
  store: Data<Store>,
  qs: Query<QueryString>,
) -> HttpResponse {
  let skip = qs.skip.unwrap_or(0);
  let limit = qs.limit.unwrap_or(100);

  exec_basic_db_read_endpoint(
    Arc::clone(&store.neo4j),
    Box::new(qs.clone().into_inner()),
    Box::new(move || {
      read_events_by_category(qs.category.clone(), skip, limit)
    })
  ).await
}
