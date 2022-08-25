use std::{
  rc::Rc,
};
use actix_web::{web};
use api_helpers::{
  middleware::auth::AuthnMiddlewareFactory,
};
use super::{
  create_sell_listing,
  create_buy_listing,
  fill_sell_listing,
};

pub fn config(authn_middleware: Rc<AuthnMiddlewareFactory>) -> impl FnOnce(&mut web::ServiceConfig) {
  move |cfg: &mut web::ServiceConfig| {
    cfg.service(
      web::resource("/{listing_account}/sell")
      .wrap(Rc::clone(&authn_middleware))
      .route(web::post().to(create_sell_listing::exec))
    );

    cfg.service(
      web::resource("/{listing_account}/buy")
      .wrap(Rc::clone(&authn_middleware))
      .route(web::post().to(create_buy_listing::exec))
    );


    cfg.service(
      web::resource("/{listing_account}/fill-sell")
      .wrap(Rc::clone(&authn_middleware))
      .route(web::post().to(fill_sell_listing::exec))
    );
  }
}
