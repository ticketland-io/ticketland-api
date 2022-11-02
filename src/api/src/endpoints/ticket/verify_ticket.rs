use std::sync::Arc;
use serde::{Deserialize};
use api_helpers::services::http::bad_request_error;
use actix_web::{
  web::{Data, Json, Path},
  HttpResponse,
};
use crate::{
  utils::store::Store,
};
use ticket_verification::verifier::{
  verify_ticket
};

#[derive(Deserialize)]
pub struct Body {
  pub event_id: String,
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
) -> HttpResponse {
  verify_ticket(
    Arc::clone(&store.rpc_client),
    Arc::clone(&store.neo4j),
    store.config.ticket_verifier_priv_key.clone(),
    &body.event_id,
    &body.code_challenge,
    &params.ticket_metadata,
    &body.ticket_owner_pubkey,
    &body.sig,
  ).await
  .map(|response| HttpResponse::Ok().json(response))
  .unwrap_or_else(|_| bad_request_error())
}
