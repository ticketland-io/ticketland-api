use actix_web::{web, HttpResponse};
use serde::{Deserialize};
use api_helpers::middleware::auth::AuthData;
use crate::{
  utils::store::Store,
};

#[derive(Deserialize)]
pub struct Body {
  name: String,
  title: String,
}

pub async fn exec(
  store: web::Data<Store>,
  body: web::Json<Body>,
  auth: AuthData,
) -> HttpResponse {
  HttpResponse::Ok().finish()
}
