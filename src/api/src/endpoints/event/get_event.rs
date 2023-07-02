use actix_web::{web, HttpResponse};
use eyre::Result;
use ticketland_core::error::Error;
use api_helpers::services::http::create_read_response;
use ticketland_data::models::event::Location;
use crate::{
  utils::store::Store,
};
use super::common::EventParams;


pub async fn exec(
  store: web::Data<Store>,
  params: web::Path<EventParams>,
) -> Result<HttpResponse, Error> {
  let v = Some(serde_json::from_str::<Location>(
"{\"name\": \"\", \"latitude\": 1.1, \"longitude\": 1}"
    )?);
  println!("v: {:?}", v);

  let event_id = params.event_id.clone();
  let mut postgres = store.pg_pool.connection().await?;
  let result = postgres.read_event_with_ticket_types(event_id, false).await;

  create_read_response(result, 0, 1)
}
