use actix_web::{
  web::{Data},
  HttpResponse,
};
use crate::{
  utils::store::Store,
};

pub async fn exec(store: Data<Store>) -> HttpResponse {
  let redirect_url = format!("{}/canva/auth", store.config.ticketland_dapp);

  HttpResponse::Found()
    .append_header(("Location", redirect_url.as_str()))
    .finish()
}
