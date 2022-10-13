use std::sync::Arc;
use futures_util::TryFutureExt;
use serde::{Serialize};
use stripe::{
  Account, AccountLink, AccountLinkType, AccountType, Client, CreateAccount,
  CreateAccountCapabilities, CreateAccountCapabilitiesCardPayments,
  CreateAccountCapabilitiesTransfers, CreateAccountLink,
};
use common_data::{
  helpers::{send_read, send_write},
  models::stripe_account::{StripeAccount},
  repositories::stripe::{
    read_stripe_user,
    create_account_link,
  }
};
use ticketland_core::error::Error;
use crate::utils::store::Store;

#[derive(Serialize)]
pub struct Response {
  pub link: String,
}

pub async fn create_link(store: Arc<Store>, uid: String) -> Result<String, Error> {
  let (query, db_query_params) = read_stripe_user(uid.clone());
  let neo4j = Arc::clone(&store.neo4j);
  let ticketland_dapp = store.config.ticketland_dapp.clone();
  let uid_copy = uid.clone();

  send_read(
    Arc::clone(&neo4j),
      query,
      db_query_params,
    )
    .and_then(|result| {
      async move {
        // if no account link exist we would need to create one using Stripe API
        if result.0.len() > 0 {
          Ok((false, TryInto::<StripeAccount>::try_into(result).unwrap().account_link.clone()))
        } else {
          create_stripe_account_link(
            store.config.stripe_key.clone(),
            uid_copy.clone(),
            ticketland_dapp,
          )
          .await
          .map(|account_link| (true, account_link.url))
        }
      }
    })
    .and_then(|(should_store, account_link)| {
      async move {
        if should_store {
          // We need to store the newly created 
          let (query, db_query_params) = create_account_link(uid.clone(), account_link.clone());
      
          send_write(
            Arc::clone(&neo4j),
            query,
            db_query_params,
          )
          .await
          .map(|_| account_link.clone())
        } else {
          Ok(account_link.clone())
        }
      }
    })
    .await
}

pub async fn refresh_link(store: Arc<Store>, uid: String) -> Result<String, Error> {
  let neo4j = Arc::clone(&store.neo4j);
  let ticketland_dapp = store.config.ticketland_dapp.clone();
  let uid_copy = uid.clone();

  create_stripe_account_link(
    store.config.stripe_key.clone(),
    uid_copy.clone(),
    ticketland_dapp,
  )
  .and_then(|account_link| {
    async move {
      let (query, db_query_params) = create_account_link(uid.clone(), account_link.url.clone());
      
      send_write(
        Arc::clone(&neo4j),
        query,
        db_query_params,
      )
      .await
      .map(|_| account_link.url.clone())
    }
  })
  .await
}

pub async fn create_stripe_account_link(
  secret_key: String,
  uid: String,
  ticketland_dapp: String,
) -> Result<AccountLink, Error> {
  let client = Client::new(secret_key);
  let account = Account::create(
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
      ..Default::default()
    },
  )
  .await
  .map_err(|error| Into::<Error>::into(format!("Stripe Error {:?}", error).as_str()))?;

  AccountLink::create(
    &client,
    CreateAccountLink {
        account: account.id.clone(),
        type_: AccountLinkType::AccountOnboarding,
        collect: None,
        expand: &[],
        refresh_url: Some(format!("{}/stripe/refresh-url?uid={}", &ticketland_dapp, &uid).as_str()),
        return_url: Some(format!("{}/stripe/return-url", &ticketland_dapp).as_str()),
    },
  )
  .await
  .map_err(|error| format!("Stripe Error {:?}", error).as_str().into())
}
