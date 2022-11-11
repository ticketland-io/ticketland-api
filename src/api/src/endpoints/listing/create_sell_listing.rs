use std::sync::Arc;
use serde::{Deserialize};
use actix_web::{
  web::{Data, Path, Json},
  HttpResponse,
};
use api_helpers::{
  services::http::internal_server_error,
  middleware::auth::AuthData,
};
use ticketland_core::error::Error;
use crate::{
  utils::store::Store,
};
use super::common::ListingParams;

#[derive(Deserialize)]
pub struct Body {
  ticket_nft: String,
  ask_price: i64,
}

pub async fn exec(
  store: Data<Store>,
  auth: AuthData,
  body: Json<Body>,
  params: Path<ListingParams>,
) -> HttpResponse {
  let (query, db_query_params) = create_sell_listing(
    auth.user.local_id.clone(),
    body.ticket_nft.clone(),
    params.listing_account.clone(),
    body.ask_price.clone(),
  );

  send_write(
    Arc::clone(&store.neo4j),
    query,
    db_query_params,
  ).await
  .map(|_| HttpResponse::Created().finish())
  .unwrap_or_else(|error: Error| internal_server_error(Some(error)))
}
