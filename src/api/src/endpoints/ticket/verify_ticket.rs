use std::sync::Arc;
use serde::{Deserialize};
use actix_web::{
  web::{Data, Json, Path},
  HttpResponse,
};
use ticketland_core::error::Error;
use crate::{
  utils::store::Store,
};
use ticket_verification::verifier::verify_ticket;

#[derive(Deserialize)]
pub struct Body {
  pub event_id: String,
  pub ticket_metadata: String,
  pub code_challenge: String,
  pub ticket_owner_pubkey: String,
  pub sig: String,
}

#[derive(Deserialize)]
pub struct Params {
  pub ticket_nft: String,
}

pub async fn exec(
  store: Data<Store>,
  body: Json<Body>,
  params: Path<Params>,
) -> Result<HttpResponse, Error> {
  let server_sig = verify_ticket(
    Arc::clone(&store.rpc_client),
    &store.pg_pool,
    &store.redis_pool,
    Arc::clone(&store.redlock),
    store.config.ticket_verifier_priv_key.clone(),
    &body.event_id,
    &body.code_challenge,
    &body.ticket_metadata,
    &body.ticket_owner_pubkey,
    &body.sig,
  ).await?;

  store.set_attended_queue
  .on_set_attended(body.event_id.to_owned(), params.ticket_nft.to_owned())
  .await?;

  let mut postgres = store.pg_pool.connection().await?;
  postgres.update_attended(params.ticket_nft.to_owned()).await?;
  
  Ok(HttpResponse::Ok().json(server_sig))
}
