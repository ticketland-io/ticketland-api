use stripe::{
  Account, AccountLink, AccountLinkType, AccountType, Client, CreateAccount,
  CreateAccountCapabilities, CreateAccountCapabilitiesCardPayments,
  CreateAccountCapabilitiesTransfers, CreateAccountLink,
};
use ticketland_core::error::Error;

pub async fn create_account_link(secret_key: String) -> Result<AccountLink, Error> {
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
        refresh_url: Some("https://ticketland-api.loophole.site/stripe/webhooks/refresh-url"),
        return_url: Some("https://ticketland-api.loophole.site/stripe/webhooks/return-url"),
    },
  )
  .await
  .map_err(|error| format!("Stripe Error {:?}", error).as_str().into())
}
