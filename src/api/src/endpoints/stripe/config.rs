use std::{
  rc::Rc,
};
use actix_web::{web};
use api_helpers::{
  middleware::auth::AuthnMiddlewareFactory,
};
use super::{
  create_account_link,
  refresh_link,
  webhooks,
  create_checkout_session,
  create_fill_listing_checkout_session,
};

pub fn config(authn_middleware: Rc<AuthnMiddlewareFactory>) -> impl FnOnce(&mut web::ServiceConfig) {
  move |cfg: &mut web::ServiceConfig| {
    cfg.service(
      web::resource("/account-links")
      .wrap(Rc::clone(&authn_middleware))
      .route(web::get().to(create_account_link::exec))
    );

    cfg.service(
      web::resource("/refresh-url")
      .wrap(Rc::clone(&authn_middleware))
      .route(web::get().to(refresh_link::exec))
    );

    cfg.service(
      web::resource("/webhooks")
      .route(web::post().to(webhooks::exec))
    );

    cfg.service(
      web::resource("/checkouts/primary")
      .wrap(Rc::clone(&authn_middleware))
      .route(web::post().to(create_checkout_session::exec))
    );

    cfg.service(
      web::resource("/checkouts/secondary")
      .wrap(Rc::clone(&authn_middleware))
      .route(web::post().to(create_fill_listing_checkout_session::exec))
    );
  }
}
