use std::{
  rc::Rc,
};
use actix_web::{web};
use api_helpers::{
  middleware::auth::AuthnMiddlewareFactory,
};
use super::{
  create_sell_listing
};

pub fn config(authn_middleware: Rc<AuthnMiddlewareFactory>) -> impl FnOnce(&mut web::ServiceConfig) {
  move |cfg: &mut web::ServiceConfig| {
    cfg.service(
      web::resource("/{event_id}/sell-listings/{listing_account}}")
      .wrap(Rc::clone(&authn_middleware))
      .route(web::post().to(create_sell_listing::exec))
    );
  }
}
