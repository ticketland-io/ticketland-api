use eyre::Result;
use actix_web::{
  web::{Data, Path},
  HttpResponse,
};
use ticketland_core::error::Error;
use api_helpers::middleware::auth::AuthData;
use crate::utils::store::Store;
use super::common::EventParams;

pub async fn exec(
  store: Data<Store>,
  auth: AuthData,
  params: Path<EventParams>,
) -> Result<HttpResponse, Error> {
  let mut postgres = store.pg_pool.connection().await?;

  postgres.delete_draft_event(
    auth.user.local_id.clone(),
    params.event_id.clone(),
  ).await?;

  Ok(HttpResponse::Ok().finish())
}
