use actix_web::{web, HttpResponse};
use api_helpers::{
  services::{
    data::QueryString,
    http::create_read_response,
  },
  middleware::auth::AuthData,
};
use crate::{
  utils::store::Store,
};

pub async fn exec(
  store: web::Data<Store>,
  qs: web::Query<QueryString>,
  auth: AuthData,
) -> HttpResponse {
  let mut postgres = store.postgres.lock().unwrap();
  let result = postgres.read_canva_designs(auth.user.local_id.clone()).await;

  create_read_response(result, None, None)
}
