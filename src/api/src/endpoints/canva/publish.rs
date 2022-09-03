use serde::{Serialize, Deserialize};
use actix_web::{
  web::{Data, Json},
  HttpResponse,
};
use crate::{
  utils::store::Store,
};

#[derive(Deserialize)]
pub struct Asset {
  name: Option<String>,
  #[serde(rename = "type")]
  file_type: Option<String>,
  url: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Body {
  message: Option<String>,
  parent: Option<String>,
  user: Option<String>,
  brand: Option<String>,
  label: Option<String>,
  assets: Vec<Asset>,
  design_id: Option<String>,
}

#[derive(Serialize)]
pub struct Response {
  #[serde(rename = "type")]
  pub result_type: String,
}

pub async fn exec(
  _store: Data<Store>,
  body: Json<Body>,
) -> HttpResponse {
  let ticket_design_url = body.assets.get(0).unwrap().url.clone();

  // TODO: Store the ticket_design_url in the db. This will be added to a list of user ticket designs.
  // Later on when user created a new ticket, she can choose one of these designs.
  // The ticket_design_url is a link to S3 bucket on Canva AWS

  HttpResponse::Ok().json(Response {
    result_type: "SUCCESS".to_owned()
  })
}
