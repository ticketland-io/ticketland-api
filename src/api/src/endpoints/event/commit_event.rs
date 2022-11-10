use std::sync::Arc;
use actix_web::{
  web::{Data, Path},
  HttpResponse,
};
use ticketland_core::{
  error::Error,
};
use api_helpers::{
  services::http::internal_server_error,
  middleware::auth::AuthData,
};
use ticketland_data::{
  helpers::{send_read},
  models::event::Event,
  repositories::event::{read_event},
};
use crate::{
  utils::store::Store,
};
use super::common::EventParams;

pub async fn exec(
  store: Data<Store>,
  _auth: AuthData,
  params: Path<EventParams>,
) -> HttpResponse {
  let event_id = params.event_id.clone();
  let (query, db_query_params) = read_event(event_id);

  let event = send_read(Arc::clone(&store.neo4j), query, db_query_params)
  .await
  .map(TryInto::<Event>::try_into)
  .unwrap_or_else(|error: Error| Err(error));

  if let Err(error) = event {
    return internal_server_error(Some(error))
  }
  
  let event = event.unwrap();
  
  store.new_event_queue
  .new_event(event.event_id, event.file_type)
  .await
  .map(|_| HttpResponse::Ok().finish())
  .unwrap_or_else(|error| internal_server_error(Some(error.root_cause())))
}
