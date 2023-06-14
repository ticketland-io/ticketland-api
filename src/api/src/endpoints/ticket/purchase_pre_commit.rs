use std::sync::Arc;
use serde::{Serialize, Deserialize};
use eyre::Result;
use actix_web::{
  web::{Data, Json},
  HttpResponse,
};
use solana_web3_rust::utils::pubkey_from_str;
use ticketland_core::error::Error;
use api_helpers::{middleware::auth::AuthData};
use program_artifacts::{ticket_nft::pda as ticket_nft_pda, event_registry::account_data::EventId};
use crate::{
  utils::store::Store,
  services::{
    ticket_purchase_pre_commit::store_ticket_purchase_pre_commit,
    ticket_availability::get_next_seat_index,
  },
};

#[derive(Serialize)]
pub struct Response {
  pub seat_index: u32,
  pub ticket_nft: String,
  pub ticket_metadata: String,
  pub seat_name: String,
}

#[derive(Deserialize)]
pub struct Body {
  event_id: String,
  ticket_nft: Option<String>,
  ticket_metadata: Option<String>,
  seat_index: Option<u32>,
  seat_name: Option<String>,
  ticket_type_index: u8,
}

fn has_all_optional_fields(body: &Json<Body>) -> bool {
  body.ticket_nft.is_some()
  && body.ticket_metadata.is_some()
  && body.seat_index.is_some()
  && body.seat_name.is_some()
}

pub async fn exec(
  store: Data<Store>,
  auth: AuthData,
  body: Json<Body>,
) -> Result<HttpResponse, Error> {
  let event_id = EventId(body.event_id.clone());
  let ticket_nft;
  let ticket_metadata;
  let seat_index;
  let seat_name;

  if has_all_optional_fields(&body) {
    seat_index = body.seat_index.unwrap();
    seat_name = body.seat_name.as_ref().unwrap().clone();
    ticket_nft = body.ticket_nft.as_ref().unwrap().clone();
    ticket_metadata = body.ticket_metadata.as_ref().unwrap().clone();
  } else {
    seat_index = get_next_seat_index(
      &store.pg_pool,
      &store.redis_pool,
      Arc::clone(&store.rpc_client),
      store.config.ticket_sale_program_state,
      &event_id,
      body.ticket_type_index,
    )
    .await?;

    seat_name = seat_index.to_string();

    ticket_nft = ticket_nft_pda::ticket_nft(
      &store.config.ticket_nft_program_state,
      seat_index,
      &event_id.val(),
      body.ticket_type_index,
    )
    .0
    .to_string();

    ticket_metadata = ticket_nft_pda::ticket_metadata(
      &store.config.ticket_nft_program_state,
      &pubkey_from_str(&ticket_nft)?,
    )
    .0
    .to_string();
  }

  store_ticket_purchase_pre_commit(
    Arc::clone(&store),
    auth.user.local_id.clone(),
    event_id.db_val(),
    ticket_nft.clone(),
    ticket_metadata.clone(),
    seat_index,
    seat_name.clone(),
    body.ticket_type_index
  )
  .await
  .map(|_| HttpResponse::Created().json(Response {
    seat_index,
    seat_name,
    ticket_nft,
    ticket_metadata,
  }))
  .map_err(|err| err.into())
}
