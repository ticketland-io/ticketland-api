use std::sync::Arc;
use serde::{Serialize, Deserialize};
use eyre::Result;
use actix_web::{
  web::{Data, Json, Path},
  HttpResponse,
};
use ticketland_core::error::Error;
use api_helpers::{middleware::auth::AuthData};
use ticketland_data::models::nft::NewTicketTypeNft;
use crate::utils::store::Store;

#[derive(Deserialize)]
pub struct Body {
  ticket_type_nfts: Vec<NewTicketTypeNft>,
}

pub async fn exec(
  store: Data<Store>,
  _auth: AuthData,
  body: Json<Body>,
) -> Result<HttpResponse, Error> {
  // TODO: check that request sender is owner of the cnt in which the nfts are contained
  let mut postgres = store.pg_pool.connection().await?;

  postgres.upsert_claimed_nfts(body.ticket_type_nfts.clone())
  .await
  .map(|_| HttpResponse::Created().finish())
  .map_err(|err| err.into())
}
