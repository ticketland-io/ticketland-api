use std::sync::Arc;
use serde::{Deserialize};
use actix_web::{
  web::{Data, Json},
  HttpResponse,
};
use ticketland_core::{
  error::Error,
};
use api_helpers::{
  services::{
    http::internal_server_error,
  },
  middleware::auth::AuthData,
};
use common_data::{
  helpers::{send_read},
  models::{canva_account::CanvaAccount},
  repositories::account::{create_canva_user},
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
) -> HttpResponse {
  let (query, db_query_params) = create_canva_user(auth.user.local_id.clone(), body.canva_uid.clone());
  
  send_read(
    Arc::clone(&store.neo4j),
    query,
    db_query_params,
  ).await
  .map(|result| {
    let canva_account = TryInto::<CanvaAccount>::try_into(result);

    if let Err(error) = canva_account {
      return internal_server_error(Some(error))
    }

    HttpResponse::Created().finish()
  })
  .unwrap_or_else(|error: Error| internal_server_error(Some(error)))
}
