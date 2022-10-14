use std::borrow::Borrow;
use actix_web::{
	web::{Data, Bytes},
	HttpRequest,
	HttpResponse,
};
use api_helpers::{
  services::{
    http::get_header_value,
  },
};
use stripe::{
	EventObject,
	EventType,
	Webhook,
	WebhookError
};
use crate::{
  utils::store::Store,
};

pub async fn exec(store: Data<Store>, req: HttpRequest, payload: Bytes) -> HttpResponse {
	handle_webhook(store, req, payload).unwrap();
	HttpResponse::Ok().finish()
}

pub fn handle_webhook(
	store: Data<Store>,
	req: HttpRequest,
	payload: Bytes
) -> Result<(), WebhookError> {
	let payload_str = std::str::from_utf8(payload.borrow()).unwrap();
	let stripe_signature = get_header_value(&req, "Stripe-Signature").unwrap_or_default();

	if let Ok(event) = Webhook::construct_event(payload_str, stripe_signature, &store.config.stripe_webhook_key) {
		match event.event_type {
			EventType::AccountUpdated => {
				if let EventObject::Account(account) = event.data.object {
					handle_account_updated(account)?;
				}
			}
			EventType::CheckoutSessionCompleted => {
				if let EventObject::CheckoutSession(session) = event.data.object {
					handle_checkout_session(session)?;
				}
			}
			_ => {
				println!("Unknown event encountered in webhook: {:?}", event.event_type);
			}
		}
	} else {
		println!("Failed to construct webhook event, ensure your webhook secret is correct.");
	}

	Ok(())
}

fn handle_account_updated(account: stripe::Account) -> Result<(), WebhookError> {
	println!("Received account updated webhook for account: {:?}", account.id);
	Ok(())
}

fn handle_checkout_session(session: stripe::CheckoutSession) -> Result<(), WebhookError> {
	println!("Received checkout session completed webhook with id: {:?}", session.id);
	Ok(())
}
