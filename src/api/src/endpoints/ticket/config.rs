use std::{
  rc::Rc,
};
use actix_web::{web};
use api_helpers::{
  middleware::auth::AuthnMiddlewareFactory,
};
use super::{
  save_user_ticket,
  get_user_tickets,
  verify_ticket,
};

pub fn config(authn_middleware: Rc<AuthnMiddlewareFactory>) -> impl FnOnce(&mut web::ServiceConfig) {
  move |cfg: &mut web::ServiceConfig| {
    cfg.service(
      web::resource("")
      .wrap(Rc::clone(&authn_middleware))
      .route(web::post().to(save_user_ticket::exec))
      .route(web::get().to(get_user_tickets::exec))
    );

    cfg.service(
      web::resource("{ticket_nft}/verifications")
      .wrap(Rc::clone(&authn_middleware))
      .route(web::post().to(verify_ticket::exec))
    );
  }
}
