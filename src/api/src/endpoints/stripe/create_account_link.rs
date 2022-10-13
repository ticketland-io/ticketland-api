use std::sync::Arc;
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

pub async fn exec(
  store: Data<Store>,
  auth: AuthData,
) -> HttpResponse {
  let (query, db_query_params) = read_stripe_user(auth.user.local_id.clone());
  
  let stripe_account = send_read(
    Arc::clone(&store.neo4j),
    query,
    db_query_params,
  )
  .await
  .map(|result| TryInto::<StripeAccount>::try_into(result).unwrap());
  
  let link = if let Err(_) = stripe_account {
    stripe::create_account_link(
      store.config.stripe_key.clone()
    )
    .and_then(|account_link| {
      let (query, db_query_params) = create_account_link(auth.user.local_id.clone(), account_link.url.clone());
      
      async move {
        let url = account_link.url.clone();

        send_write(
          Arc::clone(&store.neo4j),
          query,
          db_query_params,
        )
        .await
        .map(|_| url)
      }
    })
    .await
  } else {
    Ok(stripe_account.unwrap().account_link)
  };

  link
  .map(|link| {
    HttpResponse::Found()
    .append_header(("Location", link))
    .finish()
  })
  .unwrap_or_else(|error: Error| internal_server_error(Some(error)))
}
