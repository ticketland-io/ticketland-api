use std::sync::Arc;
use serde::{Serialize, Deserialize};
use actix_web::{
  web::{Data, Json},
  HttpResponse,
};
use common_data::{
  helpers::{send_read},
  models::account::{Account},
  repositories::account::{read_account_by_canva_id},
};
use crate::{
  utils::store::Store,
};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Body {
  user: String,
  // brand: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ErrorResponse {
  #[serde(rename = "type")]
  pub result_type: String,
  pub error_code: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SuccessResponse {
  #[serde(rename = "type")]
  pub result_type: String,
  pub labels: Vec<String>,
}

// Check the details here https://docs.developer.canva.com/apps/extensions/publish-extensions/authentication#step-4-redirect-the-user-back-to-canva
pub async fn exec(
  store: Data<Store>,
  body: Json<Body>,
) -> HttpResponse {
  let (query, db_query_params) = read_account_by_canva_id(body.user.clone());
  
  send_read(
    Arc::clone(&store.neo4j),
    query,
    db_query_params,
  ).await
  .map(|result| {
    let account = TryInto::<Account>::try_into(result);

    if let Err(_) = account {
      return HttpResponse::Ok().json(ErrorResponse {
        result_type: "ERROR".to_owned(),
        error_code: "CONFIGURATION_REQUIRED".to_owned(),
      })
    }

    HttpResponse::Ok().json(SuccessResponse {
      result_type: "SUCCESS".to_owned(),
      labels: vec!["PUBLISH".to_owned()],
    })
  })
  .unwrap_or_else(|_| {
    return HttpResponse::Ok().json(ErrorResponse {
      result_type: "ERROR".to_owned(),
      error_code: "CONFIGURATION_REQUIRED".to_owned(),
    })
  })
}
