use std::{
  sync::Arc,
  str::FromStr,
};
use futures_util::TryFutureExt;
use serde::{Serialize};
use stripe::{
  Account, AccountLink, AccountLinkType, AccountType, Client, CreateAccount,
  CreateAccountCapabilities, CreateAccountCapabilitiesCardPayments,
  CreateAccountCapabilitiesTransfers, CreateAccountLink, AccountLinkCollect,
  AccountId, AccountSettingsParams, PayoutSettingsParams, TransferScheduleParams,
  TransferScheduleInterval, CheckoutSession, Customer, CreateCustomer,
};
use common_data::{
  helpers::{send_read, send_write},
  models::stripe_account::{StripeAccount, self},
  repositories::{
    account::read_account,
    event::read_event_organizer_account,
    stripe::{
      read_stripe_user,
      upsert_account_link,
    },
  }
};
use ticketland_core::error::Error;
use crate::utils::store::Store;

#[derive(Serialize)]
pub struct Response {
  pub link: Option<String>,
}

#[derive(Serialize)]
pub struct CheckoutSessionResponse {
  pub session_id: String,
}

pub async fn create_link(store: Arc<Store>, uid: String) -> Result<String, Error> {
  let neo4j = Arc::clone(&store.neo4j);
  let ticketland_dapp = store.config.ticketland_dapp.clone();
  let uid_copy = uid.clone();

  create_stripe_account(store.config.stripe_key.clone())
  .and_then(|account| {
    async move {
      create_stripe_account_link(
        store.config.stripe_key.clone(),
        account.id.clone(),
        uid_copy.clone(),
        ticketland_dapp,
      )
      .await
      .map(|account_link| (account.id.clone(), account_link.url))
    }
  })
  .and_then(|(stripe_uid, account_link)| {
    async move {
      let (query, db_query_params) = upsert_account_link(
        uid.clone(),
        stripe_uid.to_string(),
        account_link.clone()
      );
  
      send_write(
        Arc::clone(&neo4j),
        query,
        db_query_params,
      )
      .await
      .map(|_| account_link.clone())
    }
  })
  .await
}

pub async fn refresh_link(store: Arc<Store>, uid: String) -> Result<String, Error> {
  let neo4j = Arc::clone(&store.neo4j);
  let (query, db_query_params) = read_stripe_user(uid.clone());
  let ticketland_dapp = store.config.ticketland_dapp.clone();
  let uid_copy = uid.clone();

  let stripe_account = send_read(Arc::clone(&neo4j), query, db_query_params).await?;
  let stripe_account = TryInto::<StripeAccount>::try_into(stripe_account).unwrap();
  let stripe_uid = stripe_account.stripe_uid.clone();

  let account_link = create_stripe_account_link(
    store.config.stripe_key.clone(),
    AccountId::from_str(&stripe_uid.clone()).unwrap(),
    uid_copy.clone(),
    ticketland_dapp,
  ).await?;

  let (query, db_query_params) = upsert_account_link(uid.clone(), stripe_uid, account_link.url.clone());
      
  send_write(Arc::clone(&neo4j), query, db_query_params,)
  .await
  .map(|_| account_link.url.clone())
}

pub async fn create_stripe_account(secret_key: String,) -> Result<Account, Error>  {
  let client = Client::new(secret_key);
  
  // We need to create a manual payyout schedule. The reason is that buying a ticket requires two steps.
  // We need to first charge user's card and then send a tx to the blockchain to mint the ticket.
  // However, there are no atomicity guarantees here. For example, we might charge user's card and then realize
  // that the ticket has already been purchased by someone else i.e. race condition. To avoid that we can essentially
  // revert the payment by refunding the original account if something like that happens. In the happy path scenario
  // we would release the payment to the event organizers bank account after a ticket is successfully minted.
  // For more info check https://stripe.com/docs/connect/manual-payouts
  Account::create(
    &client,
    CreateAccount {
      type_: Some(AccountType::Express),
      capabilities: Some(CreateAccountCapabilities {
        card_payments: Some(CreateAccountCapabilitiesCardPayments {
          requested: Some(true),
        }),
        transfers: Some(CreateAccountCapabilitiesTransfers {requested: Some(true)}),
        ..Default::default()
      }),
      settings: Some(AccountSettingsParams {
        payouts: Some(PayoutSettingsParams {
          schedule: Some(TransferScheduleParams {
            interval: Some(TransferScheduleInterval::Manual),
            ..Default::default()
          }),
          ..Default::default()
        }),
        ..Default::default()
      }),
      ..Default::default()
    },
  )
  .await
  .map_err(|error| Into::<Error>::into(format!("Stripe Error {:?}", error).as_str()))
}

pub async fn create_stripe_account_link(
  secret_key: String,
  stripe_uid: AccountId,
  uid: String,
  ticketland_dapp: String,
) -> Result<AccountLink, Error> {
  let client = Client::new(secret_key);

  AccountLink::create(
    &client,
    CreateAccountLink {
        account: stripe_uid,
        type_: AccountLinkType::AccountOnboarding,
        collect: Some(AccountLinkCollect::EventuallyDue),
        expand: &[],
        refresh_url: Some(format!("{}/stripe/refresh-url?uid={}", &ticketland_dapp, &uid).as_str()),
        return_url: Some(format!("{}/stripe/return-url", &ticketland_dapp).as_str()),
    },
  )
  .await
  .map_err(|error| format!("Stripe Error {:?}", error).as_str().into())
}

pub async fn create_checkout_session(
  store: Arc<Store>,
  buyer_uid: String,
  event_id: String,
  ticket_nft: String,
) -> Result<CheckoutSession, Error> {
  let client = Client::new(store.config.stripe_key.clone());
  let neo4j = Arc::clone(&store.neo4j);
  let (query, db_query_params) = read_account(buyer_uid.clone());
  let buyer_account = send_read(Arc::clone(&neo4j), query, db_query_params).await?;

  // TODO: we need to add name and email as well
  // TODO: do we need this https://github.com/arlyon/async-stripe/blob/master/examples/checkout.rs#L31?
  let customer = Customer::create(
    &client,
    CreateCustomer {
      description: Some(&buyer_uid.clone()),
      ..Default::default()
    },
  );

  let (query, db_query_params) = read_event_organizer_account(event_id.clone());
  let event_organizer_account = send_read(Arc::clone(&neo4j), query, db_query_params).await?;

  todo!()
}
