use std::sync::Arc;
use eyre::{Result};
use futures_util::TryFutureExt;
use chrono::{Duration};
use common_data::{
  helpers::{send_write},
  repositories::ticket::{upsert_user_ticket},
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
  let mut redis = store.redis.lock().unwrap();
  let redis_key = pending_ticket_key(&event_id, &ticket_nft);
  let store = Arc::clone(&store);

  redis.set_ex(
    &redis_key,
    &seat_index.to_string(),
    Duration::minutes(5).num_milliseconds() as usize,
  ).await?;

  let store = Arc::clone(&store);
  let (query, db_query_params) = upsert_user_ticket(
    uid.clone(),
    event_id.clone(),
    ticket_nft.clone(),
    ticket_metadata.clone(),
    seat_index,
    seat_name.clone(),
    ticket_type_index.clone(),
  );

  send_write(Arc::clone(&store.neo4j), query, db_query_params).await?;
  store.redlock.unlock(lock).await;
  
  Ok(())
}
