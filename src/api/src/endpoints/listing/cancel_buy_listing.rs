
use actix_web::{
  web::{Data, Path},
  HttpResponse,
};
use api_helpers::{
  services::http::create_write_response,
  middleware::auth::AuthData,
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
  let mut postgres = store.postgres.lock().await;
  let result = postgres.cancel_buy_listing(
    auth.user.local_id.clone(),
    params.listing_account.clone()
  ).await;

  create_write_response(result)
}
