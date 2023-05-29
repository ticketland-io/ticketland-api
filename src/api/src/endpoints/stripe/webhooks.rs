use std::{
  borrow::Borrow,
  str::FromStr,
};
use actix_web::{
  web::{Bytes, Data},
  HttpRequest, HttpResponse,
};
use stripe::{EventObject, EventType, Webhook};
use chrono::{Duration};
use eyre::{Result, Report, ContextCompat};
use solana_sdk::{
  pubkey::Pubkey,
};
use ticketland_data::models::{
  ticket::Ticket,
  ticket_onchain_account::TicketOnchainAccount,
};
use api_helpers::services::http::{
  get_header_value,
  internal_server_error
};
use program_artifacts::ticket_nft::pda as ticket_nft_pda;
use ticketland_core::async_helpers::timeout;
use ticketland_event_handler::{
  services::ticket_purchase::pending_ticket_key,
};
use crate::{
  utils::store::Store,
};
use ticketland_utils::logger::{
  interface::Logger,
  console_logger::ConsoleLogger,
};

pub async fn exec(store: Data<Store>, req: HttpRequest, payload: Bytes) -> HttpResponse {
  handle_webhook(store, req, payload)
  .await
  .map(|_| HttpResponse::Ok().finish())
  .unwrap_or_else(|error| internal_server_error(Some(error.root_cause())))
}

pub async fn handle_webhook(
  store: Data<Store>,
  req: HttpRequest,
  payload: Bytes,
) -> Result<()> {
  let payload_str = std::str::from_utf8(payload.borrow())?;
  let stripe_signature = get_header_value(&req, "Stripe-Signature").unwrap_or_default();

  if let Ok(event) = Webhook::construct_event(
    payload_str,
    stripe_signature,
    &store.config.stripe_webhook_key,
  ) {
      let event_type = event.type_;

      match event_type {
        EventType::AccountUpdated => {
          if let EventObject::Account(account) = event.data.object {
            handle_account_updated(&store, account).await?;
          }
        }
        EventType::CheckoutSessionCompleted => {
          if let EventObject::CheckoutSession(session) = event.data.object {
            handle_checkout_session(&store, session).await?;
          }
        }
        _ => {
          ConsoleLogger.error(&format!("Unknown event encountered in webhook: {:?}", event_type));

          return Err(Report::msg(format!("Unknown event encountered in webhook: {:?}", event_type)))?
        }
      }
  } else {
    ConsoleLogger.error("Failed to construct webhook event, ensure your webhook secret is correct.");

    return Err(Report::msg("Failed to construct webhook event, ensure your webhook secret is correct."))?;
  }

  Ok(())
}

async fn handle_account_updated(
  store: &Data<Store>,
  account: stripe::Account,
) -> Result<()> {
  let eventually_due = account
  .requirements
  .and_then(|requirements| requirements.eventually_due)
  .unwrap_or(vec![]);

  // If there are no pending info to be added by the conect use then `eventually_due` will be empty
  if eventually_due.len() == 0 {
    let mut postgres = store.pg_pool.connection().await?;
    postgres.update_stripe_account_status(account.id.to_string()).await?;
  }

  // return OK if the on boarding process for the connect account has not finished; that is there are
  // still pending `eventually_due` items. Stripe will be calling this webhook eveytime there is an update
  // e.g. user personal details added, identity card uploaded etc.
  Ok(())
}

async fn handle_checkout_session(store: &Data<Store>, session: stripe::CheckoutSession) -> Result<()> {
  let metadata = &session.metadata;
  let sale_type = metadata.get("sale_type").context("seat_index unavailable")?;

  match sale_type.as_str() {
    "primary" => handle_new_ticket_purchase(&store, session).await,
    "secondary" => handle_fill_sell_listing(&store, session).await,
    _ =>  Err(Report::msg("invalid sale type"))?,
  }
}

async fn handle_new_ticket_purchase(store: &Data<Store>, session: stripe::CheckoutSession) -> Result<()> {
  let metadata = session.metadata;
  let ticket_nft = metadata.get("ticket_nft").context("ticket_nft unavailable")?.to_string();
  let ticket_matadata = ticket_nft_pda::ticket_metadata(
    &store.config.ticket_nft_program_state,
    &Pubkey::from_str(&ticket_nft)?,
  ).0;
  let event_id = metadata.get("event_id").context("event_id unavailable")?;
  let redis_key = pending_ticket_key(&event_id, &ticket_nft);

  // Acquire a lock again so we update the state in Redis and Neo4j before someone else
  // tries to purchase the same ticket which the current user has already purchased via Stripe
  let _lock = store.redlock.lock(ticket_nft.as_bytes(), Duration::seconds(10).num_milliseconds() as usize).await?;
  let mut redis = store.redis_pool.connection().await?;

  let seat_index = metadata.get("seat_index").context("seat_index unavailable")?.to_string();

  timeout(
    Duration::seconds(10).num_milliseconds() as u64,
    redis.set_ex(&redis_key, &seat_index, Duration::days(1).num_milliseconds() as usize),
  ).await??;

  let buyer_uid = metadata.get("buyer_uid").context("buyer_uid unavailable")?.to_string();
  let seat_name = metadata.get("seat_name").context("seat_name unavailable")?.to_string();
  let ticket_type_index: i16 = metadata.get("ticket_type_index").context("ticket_type_index unavailable")?.parse()?;

  // Store the ticket nft in the db
  let mut postgres = store.pg_pool.connection().await?;

  let ticket_onchain_account = TicketOnchainAccount {
    ticket_nft: ticket_nft.clone(),
    ticket_metadata: ticket_matadata.to_string().clone(),
  };
  let ticket = Ticket {
    ticket_nft: ticket_nft.clone(),
    event_id: event_id.clone(),
    account_id: buyer_uid.clone(),
    created_at: None,
    ticket_type_index,
    seat_name: seat_name.clone(),
    seat_index: seat_index.parse::<i32>().unwrap(),
    attended: false,
    draft: false,
  };

  postgres.upsert_user_ticket(ticket, ticket_onchain_account).await?;

  // the ticket will ultimately be minted by another service that is handling these message
  store.ticket_purchase_queue.new_ticket_purchase(
    buyer_uid,
    event_id.clone(),
    metadata.get("sale_account").context("sale_account unavailable")?.to_string(),
    ticket_nft,
    metadata.get("recipient").context("recipient unavailable")?.to_string(),
    seat_index,
    seat_name,
  ).await
}

async fn handle_fill_sell_listing(store: &Data<Store>, session: stripe::CheckoutSession) -> Result<()> {
  let metadata = session.metadata;
  let ticket_nft = metadata.get("ticket_nft").context("ticket_nft unavailable")?.to_string();
  let event_id = metadata.get("event_id").context("event_id unavailable")?;
  let redis_key = pending_ticket_key(&event_id, &ticket_nft);

  // Acquire a lock again so we update the state in Redis and Neo4j before someone else
  // tries to purchase the same ticket which the current user has already purchased via Stripe
  let _lock = store.redlock.lock(ticket_nft.as_bytes(), Duration::seconds(10).num_milliseconds() as usize).await?;
  let mut redis = store.redis_pool.connection().await?;

  let seat_index = metadata.get("seat_index").context("seat_index unavailable")?.to_string();

  timeout(
    Duration::seconds(10).num_milliseconds() as u64,
    redis.set_ex(&redis_key, &seat_index, Duration::days(1).num_milliseconds() as usize),
  ).await??;

  let buyer_uid = metadata.get("buyer_id").context("buyer_id unavailable")?.to_string();
  let sell_listing = metadata.get("sell_listing_account").context("sell_listing_account unavailable")?.to_string();
  
  let mut postgres = store.pg_pool.connection().await?;
  postgres.fill_sell_listing(
    sell_listing.clone(),
    ticket_nft.clone(),
    buyer_uid.clone()
  ).await?;

  store.fill_sell_listing_queue.new_sell_listing(
    buyer_uid,
    event_id.clone(),
    metadata.get("sale_account").context("sale_account unavailable")?.to_string(),
    ticket_nft,
    metadata.get("recipient").context("recipient unavailable")?.to_string(),
    sell_listing,
  ).await
}
