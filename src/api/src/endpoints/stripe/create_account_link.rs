use std::sync::Arc;
use actix_web::{
  web::{Data},
  HttpResponse,
};
use ticketland_core::{
  error::Error,
};
use api_helpers::{
  middleware::auth::AuthData,
  services::{
    http::internal_server_error,
  },
};
use ticketland_data::{
  helpers::{send_read},
  models::stripe_account::{StripeAccount},
  repositories::stripe::{
    read_stripe_user,
  }
};
use crate::{
  utils::store::Store,
  services::stripe::{
    Response,
    create_link,
  },
};

pub async fn exec(
  store: Data<Store>,
  auth: AuthData,
) -> Result<HttpResponse, Error> {
  let uid = auth.user.local_id.clone();
  let (query, db_query_params) = read_stripe_user(uid.clone());
  let result = send_read(Arc::clone(&store.neo4j), query, db_query_params).await?;

  if result.0.len() > 0 {
    let stripe_account = TryInto::<StripeAccount>::try_into(result).unwrap();

    // user has already created a Stripe connect account
    if stripe_account.status == 1 {
      return Ok(HttpResponse::Ok().json(Response {link: None}))
    } else {
      // user has probably started the stripe connect on-boarding since we've already created an account link
      // If the link is expired Stripe will call the refresh url which is handles by a different endpoint.
      return Ok(HttpResponse::Ok().json(Response {link: Some(stripe_account.account_link.clone())}))
    }
  }

  // If value is None this means that there is no Stripe account in the db at the moment
  Ok(
    create_link(Arc::clone(&store), uid.clone())
    .await
    .map(|link| HttpResponse::Ok().json(Response {link: Some(link)}))
    .unwrap_or_else(|error| internal_server_error(Some(error.root_cause())))
  )
}
