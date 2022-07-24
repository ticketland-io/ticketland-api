use actix_web::{web, HttpResponse};
use serde::{Deserialize};
use api_helpers::middleware::auth::AuthData;
use crate::{
  utils::store::Store,
};

#[derive(Deserialize)]
pub struct Body {
  _name: String,
  _title: String,
}

pub async fn exec(
  _store: web::Data<Store>,
  _body: web::Json<Body>,
  _auth: AuthData,
) -> HttpResponse {
  HttpResponse::Ok().finish()
}
