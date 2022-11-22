use chrono::Duration;
use serde::{Deserialize};
use actix_web::{
  web::{Data, Json},
  HttpResponse,
};
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
  event_id: String,
  ticket_nft: String,
  ticket_metadata: String,
  sale_account: String,
  seat_index: u32,
  seat_name: String,
  ticket_type_index: u8,
}

pub async fn exec(
  store: Data<Store>,
  auth: AuthData,
  body: Json<Body>,
) -> Result<HttpResponse, Error> {
  let mut postgres = store.postgres.lock().await;
  let account = postgres.read_account_by_id(auth.user.local_id.clone()).await?;

  // 1. Update DB
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

  // 2. store the record in Redis so this ticket is considered unavailable
  let mut redis = store.redis.lock().await;
  let redis_key = pending_ticket_key(&body.event_id, &body.ticket_nft);

  redis.set_ex(
    &redis_key,
    &body.seat_index.to_string(),
    Duration::days(1).num_milliseconds() as usize,
  ).await?;

  // 3. Send new ticket purchase message to rabbitmq to execute operator purchase tx
  store.ticket_purchase_queue.new_ticket_purchase(
    auth.user.local_id.clone(),
    body.event_id.clone(),
    body.sale_account.clone(),
    body.ticket_nft.clone(),
    account.pubkey,
    body.seat_index.to_string(),
    body.seat_name.clone(),
  ).await?;

  Ok(HttpResponse::Created().finish())
}
