use std::sync::Arc;
use eyre::{Result};
use chrono::{Duration};
use ticketland_data::models::{
  ticket::Ticket,
  ticket_onchain_account::TicketOnchainAccount,
};
use ticketland_event_handler::{services::ticket_purchase::pending_ticket_key};
use crate::{utils::store::Store};

pub async fn store_ticket_purchase_pre_commit(
  store: Arc<Store>,
  uid: String,
  event_id: String,
  ticket_nft: String,
  ticket_metadata: String,
  seat_index: i32,
  seat_name: String,
  ticket_type_index: i16
) -> Result<()> {
  let lock = store
  .redlock
  .lock(
    ticket_nft.as_bytes(),
    Duration::seconds(50).num_milliseconds() as usize,
  )
  .await?;

  // stre the record in Redis so this ticket is considered unavailable
  let mut redis = store.redis.lock().unwrap();
  let redis_key = pending_ticket_key(&event_id, &ticket_nft);
  let store = Arc::clone(&store);

  redis.set_ex(
    &redis_key,
    &seat_index.to_string(),
    Duration::minutes(5).num_milliseconds() as usize,
  ).await?;

  let mut postgres = store.postgres.lock().unwrap();
  postgres.create_ticket_nft(TicketOnchainAccount {
    ticket_nft: ticket_nft.clone(),
    ticket_metadata: ticket_metadata.clone(),
  }).await?;

  postgres.upsert_user_ticket(Ticket {
    ticket_nft: ticket_nft.clone(),
    event_id: event_id.clone(),
    account_id: uid.clone(),
    created_at: None,
    ticket_type_index,
    seat_name: seat_name.clone(),
    seat_index,
    attended: false,
  }).await?;

  store.redlock.unlock(lock).await;
  
  Ok(())
}
