use serde::{Serialize, Deserialize};
use actix_web::{
  web::{Data, Json},
  HttpResponse,
};
use api_helpers::{
  services::{
    data::{exec_basic_db_write_endpoint},
  },
  middleware::auth::AuthData,
};
use common_data::{
  repositories::account::{create_canva_user},
};
use crate::{
  utils::store::Store,
};

pub async fn exec(
  _store: Data<Store>,
  auth: AuthData,
  // body: Json<Body>,
) -> HttpResponse {
  Ok(())
}
