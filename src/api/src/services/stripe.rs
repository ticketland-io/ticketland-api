use std::{
  sync::Arc,
  str::FromStr,
};
use eyre::Result;
use serde::{Serialize};
use stripe::{
  Account, AccountLink, AccountLinkType, AccountType, Client, CreateAccount,
  CreateAccountLink, AccountLinkCollect,
  AccountId, AccountSettingsParams, PayoutSettingsParams, TransferScheduleParams,
  TransferScheduleInterval,
};
use ticketland_data::{
  models::stripe_account::StripeAccount, connection::PostgresConnection,
};
use crate::utils::store::Store;

#[derive(Serialize)]
pub struct Response {
  pub link: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CheckoutSessionResponse {
  pub session_id: String,
}

pub async fn create_link(
  postgres: &mut PostgresConnection,
  ticketland_dapp: String,
  stripe_key: String,
  uid: String
  ) -> Result<String> {
  let ticketland_dapp = ticketland_dapp.clone();
  let uid_copy = uid.clone();

  let stripe_account = create_stripe_account(stripe_key.clone()).await?;
  let stripe_uid = stripe_account.id.clone();
  let account_link = create_stripe_account_link(
    stripe_key.clone(),
    stripe_uid.clone(),
    uid_copy.clone(),
    ticketland_dapp,
  ).await?;

  postgres.upsert_stripe_account(StripeAccount {
    stripe_uid: stripe_uid.to_string(),
    account_id: uid,
    created_at: None,
    account_link: Some(account_link.url.clone()),
    status: 0,
  }).await?;

  Ok(account_link.url)
}

pub async fn refresh_link(store: Arc<Store>, uid: String) -> Result<String> {
  let mut postgres = store.pg_pool.connection().await?;
  let stripe_account = postgres.read_stripe_account(uid.clone()).await?;

  let ticketland_dapp = store.config.ticketland_dapp.clone();
  let uid_copy = uid.clone();
  let stripe_uid = stripe_account.stripe_uid.clone();

  let account_link = create_stripe_account_link(
    store.config.stripe_key.clone(),
    AccountId::from_str(&stripe_uid.clone()).unwrap(),
    uid_copy.clone(),
    ticketland_dapp,
  ).await?;

  postgres.upsert_stripe_account(StripeAccount {
    stripe_uid: stripe_uid.to_string(),
    account_id: uid,
    created_at: None,
    account_link: Some(account_link.url.clone()),
    status: 0,
  }).await?;

  Ok(account_link.url)
}

pub async fn create_stripe_account(secret_key: String,) -> Result<Account>  {
  let client = Client::new(secret_key);

  // Do we need to create a manual payout schedule? The reason is that buying a ticket requires two steps.
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
      settings: Some(AccountSettingsParams {
        payouts: Some(PayoutSettingsParams {
          schedule: Some(TransferScheduleParams {
            interval: Some(TransferScheduleInterval::Daily),
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
  .map_err(Into::<_>::into)
}

pub async fn create_stripe_account_link(
  secret_key: String,
  stripe_uid: AccountId,
  uid: String,
  ticketland_dapp: String,
) -> Result<AccountLink> {
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
  .map_err(Into::<_>::into)
}
