use std::{
  rc::Rc,
};
use actix_web::{web};
use api_helpers::{
  middleware::auth::AuthnMiddlewareFactory,
};
use super::{
  get_sell_listings,
  get_buy_listings,
};

pub fn config(authn_middleware: Rc<AuthnMiddlewareFactory>) -> impl FnOnce(&mut web::ServiceConfig) {
  move |cfg: &mut web::ServiceConfig| {
    cfg.service(
      web::resource("/{uid}/listings/sells")
      .wrap(Rc::clone(&authn_middleware))
      .route(web::get().to(get_sell_listings::exec))
    );

    cfg.service(
      web::resource("/{uid}/listings/buys")
      .wrap(Rc::clone(&authn_middleware))
      .route(web::get().to(get_buy_listings::exec))
    );
  }
}
