use actix_web::{
  web::{Data, Json},
  HttpResponse,
};
use serde::Deserialize;
use api_helpers::{
  middleware::auth::AuthData,
};
use ticketland_core::error::Error;
use ticketland_data::{
  models::{
    ticket::Ticket,
    ticket_onchain_account::TicketOnchainAccount,
  }
};
use ticketland_event_handler::{
  services::ticket_purchase::pending_ticket_key,
};
use crate::{
  utils::store::Store,
};

#[derive(Deserialize)]
pub struct Body {
  pub ticket_nft: String,
  pub ticket_metadata: String,
  pub event_id: String,
  pub ticket_type_index: i16,
  pub seat_name: String,
  pub seat_index: i32,
}

pub async fn exec(
  store: Data<Store>,
  auth: AuthData,
  body: Json<Body>,
) -> Result<HttpResponse, Error> {
  // 1. Update DB
  let mut postgres = store.postgres.lock().await;

  let ticket_onchain_account = TicketOnchainAccount {
    ticket_nft: body.ticket_nft.clone(),
    ticket_metadata: body.ticket_metadata.clone(),
  };
  let ticket = Ticket {
    ticket_nft: body.ticket_nft.clone(),
    event_id: body.event_id.clone(),
    account_id: auth.user.local_id.clone(),
    created_at: None,
    ticket_type_index: body.ticket_type_index as i16,
    seat_name: body.seat_name.clone(),
    seat_index: body.seat_index as i32,
    attended: false,
    draft: false
  };

  postgres.upsert_user_ticket(ticket, ticket_onchain_account).await?;
  
  // 2. Remove ending key from Redis
  let mut redis = store.redis.lock().await;
  let redis_key = pending_ticket_key(&body.event_id, &body.ticket_nft);
  
  redis.delete(&redis_key).await?;

  Ok(HttpResponse::Created().finish())
}
