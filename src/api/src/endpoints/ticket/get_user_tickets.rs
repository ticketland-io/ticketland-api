use serde::{Deserialize};
use actix_web::{
  web::{Data, Query},
  HttpResponse,
};
use api_helpers::{
  QueryString,
  services::{
    data::QueryStringTrait,
    http::create_read_response,
  },
  middleware::auth::AuthData,
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
  // TODO: we want to return user events for all events if this is none
  let event_id = qs.event_id.clone().unwrap_or("".to_owned());
  let skip = qs.skip.unwrap_or(0);
  let limit = qs.limit.unwrap_or(100);
  let mut postgres = store.postgres.lock().unwrap();
  let result = postgres.read_user_tickets_for_event(event_id, skip, limit).await;

  create_read_response(result, qs.skip, qs.limit)
}
