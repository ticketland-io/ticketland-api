use std::sync::Arc;
use eyre::{Result, Report};
use chrono::{Duration};
use ticketland_data::models::cnt::CNT;
use ticketland_event_handler::{services::ticket_purchase::pending_ticket_key};
use crate::{utils::store::Store};

pub async fn store_ticket_purchase_pre_commit(
  store: Arc<Store>,
  uid: String,
  event_id: String,
  seat_index: u32,
  seat_name: String,
  ticket_type_index: u8
) -> Result<()> {
  let lock = store
  .redlock
  .lock(
    format!("{}:{}:{}", event_id, seat_name, seat_index).as_bytes(),
    Duration::seconds(50).num_milliseconds() as usize,
  )
  .await?;

  // stre the record in Redis so this ticket is considered unavailable
  let mut redis = store.redis_pool.connection().await?;
  let redis_key = pending_ticket_key(&event_id, &seat_index.to_string());
  let store = Arc::clone(&store);

  // Check if the ticket_nft key is in Redis; If so, then the ticket is not available
  if let Ok(_) = redis.get(&redis_key).await {
    return Err(Report::msg("Ticket not available"))
  }

  // store the record in Redis so this ticket is considered unavailable
  redis.set_ex(
    &redis_key,
    "1",
    Duration::minutes(5).num_seconds() as usize,
  ).await?;

  let mut postgres = store.pg_pool.connection().await?;
  let cnt = CNT {
    cnt_sui_address: None,
    event_id: event_id.clone(),
    account_id: uid.clone(),
    created_at: None,
    ticket_type_index: ticket_type_index as i16,
    seat_name: seat_name.clone(),
    seat_index: seat_index as i32,
    attended: false,
    draft: true,
  };

  postgres.upsert_user_cnt(cnt).await?;

  store.redlock.unlock(lock).await;
  
  Ok(())
}
