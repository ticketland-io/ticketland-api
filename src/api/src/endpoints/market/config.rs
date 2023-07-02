use std::{
  rc::Rc,
};
use actix_web::{web};
use api_helpers::{
  middleware::auth::AuthnMiddlewareFactory,
};
use super::{
  create_listing,
  get_listings,
  create_offer,
  get_offers,
  fill_listing,
  fill_offer,
  cancel_listing,
  cancel_offer,
  create_listing_pre_commit,
  create_offer_pre_commit,
  get_sold,
  get_prices,
};

pub fn config(authn_middleware: Rc<AuthnMiddlewareFactory>) -> impl FnOnce(&mut web::ServiceConfig) {
  move |cfg: &mut web::ServiceConfig| {
    cfg.service(
      web::resource("/listings/{listing_id}")
      .wrap(Rc::clone(&authn_middleware))
      .route(web::post().to(create_listing::exec))
    );

    cfg.service(
      web::resource("/listings/{listing_id}/pre-commit")
      .wrap(Rc::clone(&authn_middleware))
      .route(web::post().to(create_listing_pre_commit::exec))
    );

    cfg.service(
      web::resource("/listings")
      .route(web::get().to(get_listings::exec))
      .route(web::post().to(create_listing_pre_commit::exec).wrap(Rc::clone(&authn_middleware)))
    );

    cfg.service(
      web::resource("/offers/{offer_id}")
      .wrap(Rc::clone(&authn_middleware))
      .route(web::post().to(create_offer::exec))
    );

    cfg.service(
      web::resource("/offers/{offer_id}/pre-commit")
      .wrap(Rc::clone(&authn_middleware))
      .route(web::post().to(create_offer_pre_commit::exec))
    );

    cfg.service(
      web::resource("/offers")
      .route(web::get().to(get_offers::exec))
    );

    cfg.service(
      web::resource("/listings/{listing_id}/fills")
      .wrap(Rc::clone(&authn_middleware))
      .route(web::post().to(fill_listing::exec))
    );

    cfg.service(
      web::resource("/offers/{offer_id}/fills")
      .wrap(Rc::clone(&authn_middleware))
      .route(web::post().to(fill_offer::exec))
    );

    cfg.service(
      web::resource("/listings/{listing_id}/cancellations")
      .wrap(Rc::clone(&authn_middleware))
      .route(web::post().to(cancel_listing::exec))
    );

    cfg.service(
      web::resource("/offers/{offer_id}/cancellations")
      .wrap(Rc::clone(&authn_middleware))
      .route(web::post().to(cancel_offer::exec))
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
