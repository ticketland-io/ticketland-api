use serde::{Serialize, Deserialize};
use actix_web::{
  web::{Data, Json},
  HttpResponse,
};
use eyre::Result;
use ticketland_core::error::Error;
use ticketland_data::{
  models::canva_design::CanvaDesign,
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
) -> Result<HttpResponse, Error> {
  let asset = body.assets.get(0).unwrap().clone();
  
  store.ticket_design_upload_queue.new_design(
    body.design_id.clone(),
    asset.file_type.clone(),
    asset.url.clone(),
  ).await?;

  let mut postgres = store.postgres.lock().await;
  postgres.upsert_ticket_design(CanvaDesign {
    design_id: body.design_id.clone(),
    canva_uid: body.user.clone(),
    created_at: None,
    url: asset.url.clone(),
    name: asset.name.clone(),
    file_type: asset.file_type.clone(),
  })
  .await?;

  Ok(
    HttpResponse::Ok().json(SuccessResponse {
      result_type: "SUCCESS".to_string()
    })
  )
}
