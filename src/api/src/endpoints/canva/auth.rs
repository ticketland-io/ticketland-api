use serde::{Deserialize};
use actix_web::{
  web::{Data, Query},
  HttpResponse,
};
use crate::{
  utils::store::Store,
};

#[derive(Deserialize, Clone)]
pub struct QueryString {
  user: String,
  state: String,
}

pub async fn exec(
  store: Data<Store>,
  qs: Query<QueryString>,
) -> HttpResponse {
  let redirect_url = format!("{}/canva/auth?user={}&state={}", store.config.ticketland_dapp, qs.user, qs.state);

  HttpResponse::Found()
    .append_header(("Location", redirect_url.as_str()))
    .finish()
}
