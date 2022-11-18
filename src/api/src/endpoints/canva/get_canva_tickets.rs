use actix_web::{web, HttpResponse};
use api_helpers::{
  services::{
    http::create_read_response,
  },
  middleware::auth::AuthData,
};
use crate::{
  utils::store::Store,
};

pub async fn exec(
  store: web::Data<Store>,
  auth: AuthData,
) -> HttpResponse {
  let mut postgres = store.postgres.lock().await;
  let result = postgres.read_canva_designs(auth.user.local_id.clone()).await;

  create_read_response(result, 0, 1)
}
