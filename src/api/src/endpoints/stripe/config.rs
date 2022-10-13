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
      .route(web::get().to(refresh_link::exec))
    );
  }
}
