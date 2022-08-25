use std::sync::Arc;
use actix_web::{
  web::{Data, Path},
  HttpResponse,
};
use api_helpers::{
  services::{
    data::{exec_basic_db_write_endpoint},
  },
  middleware::auth::AuthData,
};
use common_data::{
  repositories::listing::{cancel_sell_listing},
};
use crate::{
  utils::store::Store,
};
use super::common::ListingParams;

pub async fn exec(
  store: Data<Store>,
  auth: AuthData,
  params: Path<ListingParams>,
) -> HttpResponse {
  exec_basic_db_write_endpoint(
    Arc::clone(&store.neo4j),
    Box::new(move || {
      cancel_sell_listing(
        auth.user.local_id.clone(),
        params.listing_account.clone(),
      )
    })
  ).await;

  HttpResponse::Created().finish()
}
