use std::sync::Arc;
use serde::{Serialize};
use actix_web::{
  web::{Data},
  HttpResponse,
};
use futures_util::TryFutureExt;
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
  helpers::{send_read, send_write},
  models::stripe_account::{StripeAccount},
  repositories::stripe::{
    read_stripe_user,
    create_account_link,
  }
};
use crate::{
  utils::store::Store,
  services::stripe,
};

#[derive(Serialize)]
pub struct Response {
  pub link: String,
}

pub async fn exec(
  store: Data<Store>,
  auth: AuthData,
) -> HttpResponse {
  let (query, db_query_params) = read_stripe_user(auth.user.local_id.clone());
  let neo4j  = Arc::clone(&store.neo4j);

  let link = send_read(
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
          stripe::create_account_link(
            store.config.stripe_key.clone()
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
          let (query, db_query_params) = create_account_link(auth.user.local_id.clone(), account_link.clone());
      
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
    .await;


  link
  .map(|link| {
    HttpResponse::Ok().json(Response {link})
  })
  .unwrap_or_else(|error: Error| internal_server_error(Some(error)))
}
