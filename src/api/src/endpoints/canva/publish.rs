use std::sync::Arc;
use serde::{Serialize, Deserialize};
use actix_web::{
  web::{Data, Json},
  HttpResponse,
};
use chrono::{Utc};
use common_data::{
  models::ticket_design::{TicketDesign},
  helpers::{send_write},
  repositories::design::{upsert_ticket_design},
};
use api_helpers::{
  services::http::internal_server_error,
};
use crate::{
  utils::store::Store,
};

#[derive(Deserialize)]
pub struct Asset {
  name: String,
  #[serde(rename = "type")]
  file_type: String,
  url: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Body {
  user: String,
  assets: Vec<Asset>,
  design_id: String,
}

#[derive(Serialize)]
pub struct SuccessResponse {
  #[serde(rename = "type")]
  pub result_type: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ErrorResponse {
  #[serde(rename = "type")]
  pub result_type: String,
  pub error_code: String,
}

pub async fn exec(
  store: Data<Store>,
  body: Json<Body>,
) -> HttpResponse {
  let asset = body.assets.get(0).unwrap().clone();
  
  let result = store.ticket_design_upload_queue.new_design(
    body.design_id.clone(),
    asset.file_type.clone(),
    asset.url.clone(),
  )
  .await
  .map(|_| HttpResponse::Ok().finish());

  if let Err(error) = result {
    return internal_server_error(Some(error.root_cause()))
  }

  let (query, db_query_params) = upsert_ticket_design(
    body.user.clone(),
    body.design_id.clone(),
    asset.url.clone(),
    asset.file_type.clone(),
    asset.name.clone(),
    Utc::now().timestamp(),
  );

  send_write(
    Arc::clone(&store.neo4j),
    query,
    db_query_params,
  ).await
  .map(|result| {
    if let Err(_) = TryInto::<TicketDesign>::try_into(result) {
      return HttpResponse::Ok().json(ErrorResponse {
        result_type: "ERROR".to_owned(),
        error_code: "CONFIGURATION_REQUIRED".to_owned(),
      })
    }
      
    HttpResponse::Ok().json(SuccessResponse {
      result_type: "SUCCESS".to_owned()
    })
  })
  .unwrap_or_else(|_| {
    return HttpResponse::Ok().json(ErrorResponse {
      result_type: "ERROR".to_owned(),
      error_code: "CONFIGURATION_REQUIRED".to_owned(),
    })
  })
}
