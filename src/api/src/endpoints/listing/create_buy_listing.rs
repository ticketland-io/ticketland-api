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
  repositories::listing::{create_buy_listing},
};
use crate::{
  utils::store::Store,
};
use super::common::ListingParams;

#[derive(Deserialize)]
pub struct Body {
  bid_price: i64,
  event_id: String,
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
      create_buy_listing(
        auth.user.local_id.clone(),
        body.event_id.clone(),
        params.listing_account.clone(),
        body.bid_price.clone(),
      )
    })
  ).await;

  HttpResponse::Created().finish()
}
