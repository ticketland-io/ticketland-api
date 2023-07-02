use std::{
  rc::Rc,
};
use actix_web::{web};
use api_helpers::{
  middleware::auth::AuthnMiddlewareFactory,
};
use super::{
  get_listings,
  get_offers,
};

pub fn config(authn_middleware: Rc<AuthnMiddlewareFactory>) -> impl FnOnce(&mut web::ServiceConfig) {
  move |cfg: &mut web::ServiceConfig| {
    cfg.service(
      web::resource("/{uid}/listings")
      .wrap(Rc::clone(&authn_middleware))
      .route(web::get().to(get_listings::exec))
    );

    cfg.service(
      web::resource("/{uid}/offers")
      .wrap(Rc::clone(&authn_middleware))
      .route(web::get().to(get_offers::exec))
    );
  }
}
