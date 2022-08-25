use std::sync::Arc;
use serde::{Deserialize};
use actix_web::{
  web::{Data, Query},
  HttpResponse,
};
use api_helpers::{
  impl_query_string,
  services::data::{QueryStringTrait, exec_basic_db_read_endpoint},
};
use common_data::{
  repositories::listing::read_buy_listings_for_event,
};
use crate::{
  utils::store::Store,
};

#[derive(Deserialize, Default, Clone)]
pub struct QueryString {
  pub skip: Option<u32>,
  pub limit: Option<u32>,
  pub event_id: String,
}

impl_query_string!(QueryString);

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
      read_buy_listings_for_event(qs.event_id.clone(), skip, limit)
    })
  ).await
}
