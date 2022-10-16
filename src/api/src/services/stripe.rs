use std::{
  sync::Arc,
  str::FromStr,
};
use serde::{Serialize};
use stripe::{
  Account, AccountLink, AccountLinkType, AccountType, Client, CreateAccount,
  CreateAccountCapabilities, CreateAccountCapabilitiesCardPayments,
  CreateAccountCapabilitiesTransfers, CreateAccountLink, AccountLinkCollect,
  AccountId, AccountSettingsParams, PayoutSettingsParams, TransferScheduleParams,
  TransferScheduleInterval, Customer, CreateCustomer, CreateProduct,
  Product, CreatePrice, Currency, IdOrCreate, Price, CreateCheckoutSession, CheckoutSession,
  CreateCheckoutSessionLineItems, CheckoutSessionMode, CreateCheckoutSessionPaymentIntentData,
  CreateCheckoutSessionPaymentIntentDataTransferData,
};
use common_data::{
  helpers::{send_read, send_write},
  models::stripe_account::{StripeAccount},
  repositories::{
    stripe::{
      read_stripe_user,
      upsert_account_link,
      read_event_organizer_stripe_account,
    },
  }
};
use ticketland_core::error::Error;
use crate::utils::store::Store;
use super::ticket_purchase::{
  calculate_price_and_fees,
};

#[derive(Serialize)]
pub struct Response {
  pub link: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CheckoutSessionResponse {
  pub session_id: String,
}

fn into_stripe_error(error: impl std::error::Error) -> Error {
  Into::<Error>::into(format!("Stripe Error {:?}", error).as_str())
}

pub async fn create_link(store: Arc<Store>, uid: String) -> Result<String, Error> {
  let neo4j = Arc::clone(&store.neo4j);
  let ticketland_dapp = store.config.ticketland_dapp.clone();
  let uid_copy = uid.clone();

  let stripe_account = create_stripe_account(store.config.stripe_key.clone()).await?;
  let stripe_uid = stripe_account.id.clone();
  let account_link = create_stripe_account_link(
    store.config.stripe_key.clone(),
    stripe_uid.clone(),
    uid_copy.clone(),
    ticketland_dapp,
  ).await?;

  let (query, db_query_params) = upsert_account_link(uid.clone(), stripe_uid.to_string(), account_link.url.clone());
  send_write(Arc::clone(&neo4j), query, db_query_params)
  .await
  .map(|_| account_link.url)
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
  .map_err(into_stripe_error)
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
  .map_err(into_stripe_error)
}

pub async fn create_checkout_session(
  store: Arc<Store>,
  buyer_uid: String,
  event_id: String,
  ticket_nft: String,
) -> Result<String, Error> {
  let client = Client::new(store.config.stripe_key.clone());
  let neo4j = Arc::clone(&store.neo4j);

  // TODO: we need to add name and email as well
  // let (query, db_query_params) = read_account(buyer_uid.clone());
  // let buyer_account = send_read(Arc::clone(&neo4j), query, db_query_params).await?;
  // TODO: do we need this https://github.com/arlyon/async-stripe/blob/master/examples/checkout.rs#L31?
  let customer = Customer::create(
    &client,
    CreateCustomer {
      description: Some(&buyer_uid.clone()),
      ..Default::default()
    },
  ).await
  .map_err(into_stripe_error)?;

  let product = {
    // TODO: we can additional props to the product such as url
    let product_name = format!("{}{}", &event_id, &ticket_nft);
    let create_product = CreateProduct::new(&product_name);
    
    Product::create(&client, create_product)
    .await
    .map_err(|error| Into::<Error>::into(format!("Stripe Error {:?}", error).as_str()))?
  };

  let (price, fee) = calculate_price_and_fees(&event_id).await?;

  // and add a price for it in USD
  let price = {
    // TODO: we might wnat to support multiple currencies
    let mut create_price = CreatePrice::new(Currency::USD);
    create_price.product = Some(IdOrCreate::Id(&product.id));
    create_price.unit_amount = Some(price);
    create_price.expand = &["product"];

    Price::create(&client, create_price).await.map_err(into_stripe_error)?
  };

  let (query, db_query_params) = read_event_organizer_stripe_account(event_id.clone());
  let stripe_account = send_read(Arc::clone(&neo4j), query, db_query_params).await?;
  let stripe_account = TryInto::<StripeAccount>::try_into(stripe_account).unwrap();

  let checkout_session = {
    let ticketland_dapp = store.config.ticketland_dapp.clone();
    // TODO: use the correct urls
    let cancel_url = format!("{}/stripe/cancel", &ticketland_dapp);
    let success_url = format!("{}/stripe/success", &ticketland_dapp);

    let mut params = CreateCheckoutSession::new(&cancel_url, &success_url);
    params.customer = Some(customer.id);
    params.payment_intent_data = Some(CreateCheckoutSessionPaymentIntentData {
      application_fee_amount: Some(fee),
      transfer_data: Some(CreateCheckoutSessionPaymentIntentDataTransferData {
        destination: stripe_account.stripe_uid,
        ..Default::default()  
      }),
      ..Default::default()
    });

    params.mode = Some(CheckoutSessionMode::Payment);
    params.line_items = Some(vec![CreateCheckoutSessionLineItems {
      quantity: Some(1),
      price: Some(price.id.to_string()),
      ..Default::default()
    }]);
    params.expand = &["line_items", "line_items.data.price.product"];

    CheckoutSession::create(&client, params)
    .await
    .map_err(into_stripe_error)?
  };

  Ok(checkout_session.id.to_string())
}
