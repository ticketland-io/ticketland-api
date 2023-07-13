use std::sync::Arc;
use eyre::Result;
use actix_web::{web, HttpResponse};
use actix_multipart::Multipart;
use ticketland_core::error::Error;
use api_helpers::{
  middleware::auth::AuthData, 
  services::http::create_write_response,
};
use crate::{
  utils::store::Store,
  services::event::update_event_ticket_type_nft_details,
};
use super::common::EventParams;

pub async fn exec(
  store: web::Data<Store>,
  _auth: AuthData,
  params: web::Path<EventParams>,
  payload: Multipart
) -> Result<HttpResponse, Error> {
  // TODO: check the request sender is the owner of the event
  let event_id = params.event_id.clone();

  let result = update_event_ticket_type_nft_details(
    Arc::clone(&store),
    event_id,
    payload,
  ).await;

  create_write_response(result)
}
