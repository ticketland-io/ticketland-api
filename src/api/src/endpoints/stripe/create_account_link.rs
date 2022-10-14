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
use common_data::{
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
) -> HttpResponse {
  let uid = auth.user.local_id.clone();
  let (query, db_query_params) = read_stripe_user(uid.clone());

  let result = send_read(Arc::clone(&store.neo4j), query, db_query_params)
  .await
  .map(|result| {
    if result.0.len() > 0 {
      Some(TryInto::<StripeAccount>::try_into(result).unwrap())
    } else {
      None
    }
  });

  match result {
    Ok(stripe_account) => {
      if let Some(stripe_account) = stripe_account {
        // user has already created a Stripe connect account
        if stripe_account.status == 1 {
          HttpResponse::Ok().json(Response {link: "".to_owned()})
        } else {
          // user has probably started the stripe connect on-boarding since we've already created an account link
          // If the link is expired Stripe will call the refresh url which is handles by a different endpoint.
          HttpResponse::Ok().json(Response {link: stripe_account.account_link.clone()})
        }
      } else { 
        // If value is None this means that there is no Stripe account in the db at the moment
        create_link(Arc::clone(&store), uid.clone())
        .await
        .map(|link| HttpResponse::Ok().json(Response {link}))
        .unwrap_or_else(|error: Error| internal_server_error(Some(error)))
      }
    },
    Err(error) => internal_server_error(Some(error)),
  }
}
