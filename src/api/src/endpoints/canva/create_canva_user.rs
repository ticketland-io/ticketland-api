use serde::{Deserialize};
use eyre::Result;
use actix_web::{
  web::{Data, Json},
  HttpResponse,
};
use ticketland_core::{
  error::Error,
};
use api_helpers::{
  middleware::auth::AuthData, 
  services::http::create_write_response,
};
use ticketland_data::{
  models::{canva_account::CanvaAccount},
};
use crate::{
  utils::store::Store,
};

#[derive(Deserialize)]
pub struct Body {
  canva_uid: String,
}

pub async fn exec(
  store: Data<Store>,
  auth: AuthData,
  body: Json<Body>,
) -> Result<HttpResponse, Error> {
  let mut postgres = store.pg_pool.connection().await?;
  let result = postgres.upsert_canva_account(CanvaAccount {
    canva_uid: body.canva_uid.clone(),
    account_id: auth.user.local_id.clone(),
    created_at: None,
  }).await;

  Ok(create_write_response(result))
}
