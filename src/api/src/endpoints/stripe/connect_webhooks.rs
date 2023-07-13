use std::borrow::Borrow;
use actix_web::{
  web::{Bytes, Data},
  HttpRequest, HttpResponse,
};
use stripe::{EventObject, EventType, Webhook};
use eyre::{Result, Report};
use ticketland_core::error::Error;
use api_helpers::services::http::get_header_value;
use crate::utils::store::Store;

pub async fn exec(store: Data<Store>, req: HttpRequest, payload: Bytes) -> Result<HttpResponse, Error> {
  handle_connect_webhook(store, req, payload)
  .await
  .map(|_| HttpResponse::Ok().finish())
  .map_err(|err| err.into())
}

pub async fn handle_connect_webhook(
  store: Data<Store>,
  req: HttpRequest,
  payload: Bytes,
) -> Result<()> {
  let payload_str = std::str::from_utf8(payload.borrow())?;
  let stripe_signature = get_header_value(&req, "Stripe-Signature").unwrap_or_default();

  if let Ok(event) = Webhook::construct_event(
    payload_str,
    stripe_signature,
    &store.config.stripe_connect_webhook_key,
  ) {
      let event_type = event.type_;

      match event_type {
        EventType::AccountUpdated => {
          if let EventObject::Account(account) = event.data.object {
            handle_account_updated(&store, account).await?;
          }
        }
        _ =>  Err(Report::msg(format!("Unknown event encountered in webhook: {:?}", event_type)))?
      }
  } else {
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
