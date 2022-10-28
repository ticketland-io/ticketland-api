use std::{borrow::Borrow, sync::Arc};
use actix_web::{
  web::{Bytes, Data},
  HttpRequest, HttpResponse,
};
use stripe::{EventObject, EventType, Webhook};
use chrono::{Duration};
use eyre::{Result, Report};
use api_helpers::services::http::{
  get_header_value,
  internal_server_error
};
use ticketland_core::async_helpers::timeout;
use common_data::{
  helpers::send_write,
  repositories::{
    ticket::upsert_user_ticket,
    stripe::update_stripe_account_status,
  },
};
use ticketland_event_handler::{
  services::ticket_purchase::pending_ticket_key,
};
use program_artifacts::ticket_nft::pda::ticket_metadata;
use crate::{
  utils::store::Store,
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
      match event.event_type {
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
          println!("Unknown event encountered in webhook: {:?}", event.event_type);

          return Err(Report::msg(format!("Unknown event encountered in webhook: {:?}", event.event_type)))?
        }
      }
  } else {
    println!("Failed to construct webhook event, ensure your webhook secret is correct.");

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
    let (query, db_query_params) = update_stripe_account_status(account.id.to_string());

    return send_write(Arc::clone(&store.neo4j), query, db_query_params).await.map(|_| ()).map_err(Into::<_>::into)
  }

  // return OK if the on boarding process for the connect account has not finished; that is there are
  // still pending `eventually_due` items. Stripe will be calling this webhook eveytime there is an update
  // e.g. user personal details added, identity card uploaded etc.
  Ok(())
}

async fn handle_checkout_session(store: &Data<Store>, session: stripe::CheckoutSession) -> Result<()> {
  let metadata = &session.metadata;
  let sale_type = metadata.get("sale_type").unwrap();

  match sale_type.as_str() {
    "primary" => handle_new_ticket_purchase(&store, session).await,
    "secondary" => handle_fill_sell_listing(&store, session).await,
    _ => Err(Report::msg("invalid sale type"))?,
  }
}

async fn handle_new_ticket_purchase(store: &Data<Store>, session: stripe::CheckoutSession) -> Result<()> {
  let metadata = session.metadata;
  let ticket_nft = metadata.get("ticket_nft").unwrap().to_string();
  let event_id = metadata.get("event_id").unwrap();
  let redis_key = pending_ticket_key(&event_id, &ticket_nft);

  // Acquire a lock again so we update the state in Redis and Neo4j before someone else
  // tries to purchase the same ticket which the current user has already purchased via Stripe
  let _lock = store.redlock.lock(ticket_nft.as_bytes(), Duration::seconds(10).num_milliseconds() as usize).await?;
  let mut redis = store.redis.lock().unwrap();

  timeout(
    Duration::seconds(10).num_milliseconds() as u64,
    redis.set(&redis_key, &"1"),
  ).await??;


  let buyer_uid = metadata.get("buyer_id").unwrap().to_string();
  let seat_index = metadata.get("seat_index").unwrap().to_string();
  let seat_name = metadata.get("seat_name").unwrap().to_string();

  // Store the ticket nft in the db
  let (query, db_query_params) = upsert_user_ticket(
    buyer_uid.clone(),
    event_id.clone(),
    ticket_nft.clone(),
    ticket_metadata(&store.config.ticket_nft_program_state, &ticket_nft).0.to_string(),
    seat_index.parse::<u32>().unwrap(),
    seat_name.clone(),
  );

  send_write(Arc::clone(&store.neo4j), query, db_query_params).await?;

  // the ticket will ultimately be minted by another service that is handling these message
  store.ticket_purchase_queue.new_ticket_purchase(
    buyer_uid,
    event_id.clone(),
    metadata.get("sale_account").unwrap().to_string(),
    ticket_nft,
    metadata.get("recipient").unwrap().to_string(),
    seat_index,
    seat_name,
  ).await
}

async fn handle_fill_sell_listing(store: &Data<Store>, session: stripe::CheckoutSession) -> Result<()> {
  todo!()
}
