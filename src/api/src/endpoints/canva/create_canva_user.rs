use std::sync::Arc;
use serde::{Deserialize};
use actix_web::{
  web::{Data, Json},
  HttpResponse,
};
use api_helpers::{
  middleware::auth::AuthData,
};
use common_data::{
  helpers::{send_write},
  models::{canva_account::CanvaAccount},
  repositories::account::{create_canva_user},
};
use crate::{
  utils::store::Store,
};

#[derive(Deserialize)]
pub struct Body {
  canva_uid: String,
  canva_token: String,
}

pub async fn exec(
  store: Data<Store>,
  auth: AuthData,
  body: Json<Body>,
) -> HttpResponse {
  let (query, db_query_params) = create_canva_user(auth.user.local_id.clone(), body.canva_uid.clone());
  
  let response_path = send_write(
    Arc::clone(&store.neo4j),
    query,
    db_query_params,
  ).await
  .map(|result| {
    if let Err(_) = TryInto::<CanvaAccount>::try_into(result) {
      return format!("https://canva.com/apps/configured?success=false&state={}", body.canva_token.clone())
    }

    format!("https://canva.com/apps/configured?success=true&state={}", body.canva_token.clone())
  })
  .unwrap_or_else(|_| format!("https://canva.com/apps/configured?success=false&state={}", body.canva_token.clone()));


  // https://docs.developer.canva.com/apps/extensions/publish-extensions/authentication#step-4-redirect-the-user-back-to-canva
  HttpResponse::Found()
  .append_header(("Location", response_path.as_str()))
  .finish()
}
