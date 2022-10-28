use std::sync::Arc;
use serde::{Deserialize};
use actix_web::{
  web::{Data, Path, Json},
  HttpResponse,
};
use api_helpers::{
  services::{
    data::{exec_basic_db_write_endpoint},
  },
  middleware::auth::AuthData,
};
use common_data::{
  repositories::listing::{fill_sell_listing},
};
use crate::{
  utils::store::Store,
};
use super::common::ListingParams;

#[derive(Deserialize)]
pub struct Body {
  ticket_nft: String,
}

pub async fn exec(
  store: Data<Store>,
  auth: AuthData,
  body: Json<Body>,
  params: Path<ListingParams>,
) -> HttpResponse {
  exec_basic_db_write_endpoint(
    Arc::clone(&store.neo4j),
    Box::new(move || {
      fill_sell_listing(
        auth.user.local_id.clone(),
        params.listing_account.clone(),
        body.ticket_nft.clone(),
      )
    })
  ).await;

  HttpResponse::Created().finish()
}
