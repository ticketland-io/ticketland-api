use serde::{Deserialize};
use actix_web::{
  web::{Data, Json, Path},
  HttpResponse,
};
use ticketland_core::error::Error;
use api_helpers::{
  services::http::create_write_response,
  middleware::auth::AuthData,
};
use ticketland_data::models::offer::NewOffer;
use crate::{
  utils::store::Store,
};

use super::common::OfferParams;

#[derive(Deserialize)]
pub struct Body {
  bid_price: i64,
  event_id: String,
  ticket_type_index: i16,
}

pub async fn exec(
  store: Data<Store>,
  auth: AuthData,
  body: Json<Body>,
  params: Path<OfferParams>,
) -> Result<HttpResponse, Error> {
  let mut postgres = store.pg_pool.connection().await?;
  let result = postgres.upsert_offer(NewOffer {
    offer_id: &params.offer_id,
    offer_sui_address: None,
    account_id: &auth.user.local_id,
    event_id: &body.event_id,
    ticket_type_index: body.ticket_type_index,
    bid_price: body.bid_price,
    is_open: true,
    draft: true,
  }).await;

  create_write_response(result)
}
