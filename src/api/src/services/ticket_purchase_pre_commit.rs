use std::sync::Arc;
use eyre::{Result, Report};
use chrono::{Duration};
use ticketland_data::models::{
  ticket::Ticket,
  ticket_onchain_account::TicketOnchainAccount,
};
use ticketland_utils::logger::{
  interface::Logger,
  console_logger::ConsoleLogger,
};
use ticketland_event_handler::{services::ticket_purchase::pending_ticket_key};
use crate::{utils::store::Store};

pub async fn store_ticket_purchase_pre_commit(
  store: Arc<Store>,
  uid: String,
  event_id: String,
  ticket_nft: String,
  ticket_metadata: String,
  seat_index: u32,
  seat_name: String,
  ticket_type_index: u8
) -> Result<()> {
  let lock = store
  .redlock
  .lock(
    ticket_nft.as_bytes(),
    Duration::seconds(50).num_milliseconds() as usize,
  )
  .await?;

  // stre the record in Redis so this ticket is considered unavailable
  let mut redis = store.redis_pool.connection().await?;
  let redis_key = pending_ticket_key(&event_id, &ticket_nft);
  let store = Arc::clone(&store);

  // Check if the ticket_nft key is in Redis; If so, then the ticket is not available
  if let Ok(_) = redis.get(&redis_key).await {
    ConsoleLogger.error("Ticket not available");

    return Err(Report::msg("Ticket not available"))
  }

  // store the record in Redis so this ticket is considered unavailable
  redis.set_ex(
    &redis_key,
    &seat_index.to_string(),
    Duration::minutes(5).num_milliseconds() as usize,
  ).await?;

  let mut postgres = store.pg_pool.connection().await?;
  let ticket_onchain_account = TicketOnchainAccount {
    ticket_nft: ticket_nft.clone(),
    ticket_metadata: ticket_metadata.clone(),
  };
  let ticket = Ticket {
    ticket_nft: ticket_nft.clone(),
    event_id: event_id.clone(),
    account_id: uid.clone(),
    created_at: None,
    ticket_type_index: ticket_type_index as i16,
    seat_name: seat_name.clone(),
    seat_index: seat_index as i32,
    attended: false,
    draft: true,
  };

  postgres.upsert_user_ticket(ticket, ticket_onchain_account).await?;

  store.redlock.unlock(lock).await;
  
  Ok(())
}
