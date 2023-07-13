use actix_web::{
  web::{Data, Path, Json},
  HttpResponse,
};
use eyre::Result;
use serde::Deserialize;
use ticketland_core::error::Error;
use api_helpers::middleware::auth::AuthData;
use crate::utils::store::Store;
use super::common::EventParams;

// TODO: try using ObjectId instead of String
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Body {
  pub sui_address: String,
  pub organizer_cap: String,
  pub operator_cap: String,
  pub event_nft: String,
  pub event_capacity_bitmap_address: String,
  pub ticket_type_addresses: Vec<String>,
}

// TODO: add authentication that sender is the event creator
pub async fn exec(
  store: Data<Store>,
  _auth: AuthData,
  params: Path<EventParams>,
  body: Json<Body>
) -> Result<HttpResponse, Error> {
  let event_id = params.event_id.clone();
  let mut postgres = store.pg_pool.connection().await?;
  // TODO: check the request sender is the owner of the event
  // let event = postgres.read_event(event_id.clone(), true).await?;
  // let event = event.get(0).context("event not found")?;

  postgres.commit_event(
    event_id.clone(),
    body.sui_address.clone(),
    body.organizer_cap.clone(),
    body.operator_cap.clone(),
    body.event_nft.clone(),
    body.event_capacity_bitmap_address.clone(),
    body.ticket_type_addresses.clone(),
    ).await?;

  Ok(HttpResponse::Ok().finish())
}
