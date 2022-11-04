use std::sync::Arc;
use serde::{Serialize, Deserialize};
use api_helpers::services::http::bad_request_error;
use actix_web::{
  web::{Data, Json, Path},
  HttpResponse,
};
use common_data::{repositories::ticket::update_attended, helpers::send_write};
use ticketland_core::error::Error;
use crate::{
  utils::store::Store,
};
use ticket_verification::verifier::{
  verify_ticket
};

#[derive(Deserialize)]
pub struct Body {
  pub event_id: String,
  pub ticket_nft: String,
  pub code_challenge: String,
  pub ticket_owner_pubkey: String,
  pub sig: String,
}

#[derive(Deserialize)]
pub struct Params {
  pub ticket_metadata: String,
}

pub async fn exec(
  store: Data<Store>,
  body: Json<Body>,
  params: Path<Params>,
) -> Result<HttpResponse, Error> {
  let server_sig = match verify_ticket(
    Arc::clone(&store.rpc_client),
    Arc::clone(&store.neo4j),
    store.config.ticket_verifier_priv_key.clone(),
    &body.event_id,
    &body.code_challenge,
    &params.ticket_metadata,
    &body.ticket_nft,
    &body.ticket_owner_pubkey,
    &body.sig,
  ).await {
    Ok(server_sig) => server_sig,
    Err(_) => return Ok(bad_request_error()),
  };

  // Update the db
  let (query, db_query_params) = update_attended(body.ticket_nft.to_owned());

  send_write(Arc::clone(&store.neo4j), query, db_query_params).await?;

  Ok(HttpResponse::Ok().json(Response {
    event_id: body.event_id.clone(),
    code_challenge: body.code_challenge.clone(),
    ticket_owner_pubkey: body.ticket_owner_pubkey.clone(),
    ticket_metadata: params.ticket_metadata.clone(),
    server_sig,
  }))
}
