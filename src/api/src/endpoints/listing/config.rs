use std::{
  rc::Rc,
};
use actix_web::{web};
use api_helpers::{
  middleware::auth::AuthnMiddlewareFactory,
};
use super::{
  create_sell_listing,
  get_sell_listings,
  create_buy_listing,
  get_buy_listings,
  fill_sell_listing,
  fill_buy_listing,
  cancel_sell_listing,
  cancel_buy_listing,
  create_sell_listing_pre_commit,
  create_buy_listing_pre_commit,
  get_sold,
  get_prices,
};

pub fn config(authn_middleware: Rc<AuthnMiddlewareFactory>) -> impl FnOnce(&mut web::ServiceConfig) {
  move |cfg: &mut web::ServiceConfig| {
    cfg.service(
      web::resource("/{listing_account}/sells")
      .wrap(Rc::clone(&authn_middleware))
      .route(web::post().to(create_sell_listing::exec))
    );

    cfg.service(
      web::resource("/{listing_account}/sells/pre-commit")
      .wrap(Rc::clone(&authn_middleware))
      .route(web::post().to(create_sell_listing_pre_commit::exec))
    );

    cfg.service(
      web::resource("/sells")
      .route(web::get().to(get_sell_listings::exec))
    );

    cfg.service(
      web::resource("/{listing_account}/buys")
      .wrap(Rc::clone(&authn_middleware))
      .route(web::post().to(create_buy_listing::exec))
    );

    cfg.service(
      web::resource("/{listing_account}/buys/pre-commit")
      .wrap(Rc::clone(&authn_middleware))
      .route(web::post().to(create_buy_listing_pre_commit::exec))
    );

    cfg.service(
      web::resource("/buys")
      .route(web::get().to(get_buy_listings::exec))
    );

    cfg.service(
      web::resource("/{listing_account}/sell-fills")
      .wrap(Rc::clone(&authn_middleware))
      .route(web::post().to(fill_sell_listing::exec))
    );

    cfg.service(
      web::resource("/{listing_account}/buy-fills")
      .wrap(Rc::clone(&authn_middleware))
      .route(web::post().to(fill_buy_listing::exec))
    );

    cfg.service(
      web::resource("/{listing_account}/sell-cancellations")
      .wrap(Rc::clone(&authn_middleware))
      .route(web::post().to(cancel_sell_listing::exec))
    );

    cfg.service(
      web::resource("/{listing_account}/buy-cancellations")
      .wrap(Rc::clone(&authn_middleware))
      .route(web::post().to(cancel_buy_listing::exec))
    );

    cfg.service(
      web::resource("/total-sold")
      .route(web::get().to(get_sold::exec))
    );

    cfg.service(
      web::resource("/average-prices")
      .route(web::get().to(get_prices::exec))
    );
  }
}
