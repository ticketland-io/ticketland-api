use std::{
	sync::Arc,
	borrow::Borrow,
};
use actix_web::{
	web::{Data, Bytes},
	HttpRequest,
	HttpResponse,
};
use stripe::{
	EventObject,
	EventType,
	Webhook,
};
use ticketland_core::{
  error::Error,
};
use api_helpers::{
  services::{
    http::{
			get_header_value,
			internal_server_error,
		},
  },
};
use common_data::{
  helpers::{send_write},
  repositories::stripe::{update_stripe_account_status},
};
use crate::{
  utils::store::Store,
};

pub async fn exec(store: Data<Store>, req: HttpRequest, payload: Bytes) -> HttpResponse {
	handle_webhook(store, req, payload)
	.await
	.map(|_| HttpResponse::Ok().finish())
	.unwrap_or_else(|error: Error| internal_server_error(Some(error)))
}

pub async fn handle_webhook(
	store: Data<Store>,
	req: HttpRequest,
	payload: Bytes
) -> Result<(), Error> {
	let payload_str = std::str::from_utf8(payload.borrow()).unwrap();
	let stripe_signature = get_header_value(&req, "Stripe-Signature").unwrap_or_default();

	if let Ok(event) = Webhook::construct_event(payload_str, stripe_signature, &store.config.stripe_webhook_key) {
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
				return Err(format!("Unknown event encountered in webhook: {:?}", event.event_type).as_str().into())
			}
		}
	} else {
		println!("Failed to construct webhook event, ensure your webhook secret is correct.");
		return Err("Failed to construct webhook event, ensure your webhook secret is correct.".into())
	}

	Ok(())
}

async fn handle_account_updated(store: &Data<Store>, account: stripe::Account) -> Result<(), Error> {
	let eventually_due = account.requirements
	.and_then(|requirements| requirements.eventually_due)
	.unwrap_or(vec![]);

	// If there are no pending info to be added by the conect use then `eventually_due` will be empty 
	if eventually_due.len() == 0 {
		let (query, db_query_params) = update_stripe_account_status(account.id.to_string());

		return send_write(
			Arc::clone(&store.neo4j),
			query,
			db_query_params,
		)
		.await
		.map(|_| ())
	}

	// return OK if the on boarding process for the connect account has not finished; that is there are
	// still pending `eventually_due` items. Stripe will be calling this webhook eveytime there is an update
	// e.g. user personal details added, identity card uploaded etc.
	Ok(())
}

async fn handle_checkout_session(_store: &Data<Store>, session: stripe::CheckoutSession) -> Result<(), Error> {
	println!("Received checkout session completed webhook with id: {:?}", session.id);
	Ok(())
}
