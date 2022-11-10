use std::sync::Arc;
use serde::{Deserialize};
use actix_web::{
  web::{Data, Query},
  HttpResponse,
};
use api_helpers::{
  QueryString,
  services::data::{QueryStringTrait, exec_basic_db_read_endpoint},
  middleware::auth::AuthData,
};
use ticketland_data::{
  repositories::ticket::read_user_tickets_for_event,
};
use crate::{
  utils::store::Store,
};

QueryString! {
  pub struct QueryString {
    pub event_id: Option<String>,
  }
}

pub async fn exec(
  store: Data<Store>,
  auth: AuthData,
  qs: Query<QueryString>,
) -> HttpResponse {
  let skip = qs.skip.unwrap_or(0);
  let limit = qs.limit.unwrap_or(100);
  // TODO: we want to return user events for all events if this is none
  let event_id = qs.event_id.clone().unwrap_or("".to_owned());
  
  exec_basic_db_read_endpoint(
    Arc::clone(&store.neo4j),
    Box::new(qs.clone().into_inner()),
    Box::new(move || {
      read_user_tickets_for_event(
        auth.user.local_id.clone(),
        event_id,
        skip,
        limit
      )
    })
  ).await
}
