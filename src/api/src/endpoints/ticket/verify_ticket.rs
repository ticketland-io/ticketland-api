use serde::{Deserialize};
use actix_web::{
  web::{Data, Json, Query},
  HttpResponse,
};
use crate::{
  utils::store::Store,
};

#[derive(Deserialize)]
pub struct Body {
  pub event_id: String,
  pub code_challenge: String,
}

#[derive(Deserialize)]
pub struct QueryString {
  pub ticket_nft: Option<String>,
}

pub async fn exec(
  store: Data<Store>,
  body: Json<Body>,
  qs: Query<QueryString>,
) -> HttpResponse {
  // 1. recover the signer

  // 2. check that signer is the owner of the given ticket_nft 

  todo!()
}
