use std::{
  rc::Rc,
};
use actix_web::{web};
use api_helpers::{
  middleware::auth::AuthnMiddlewareFactory,
};
use super::{
  save_user_ticket,
  get_event_tickets,
  get_user_tickets,
  verify_ticket,
  purchase_pre_commit,
  save_user_ticket_fiat,
  purchase_pre_commit_fiat,
  tickets_by_type
};

pub fn config(authn_middleware: Rc<AuthnMiddlewareFactory>) -> impl FnOnce(&mut web::ServiceConfig) {
  move |cfg: &mut web::ServiceConfig| {
    cfg.service(
      web::resource("")
      .wrap(Rc::clone(&authn_middleware))
      .route(web::post().to(save_user_ticket::exec))
      .route(web::get().to(get_event_tickets::exec))
    );
    cfg.service(
      web::resource("current-user")
      .wrap(Rc::clone(&authn_middleware))
      .route(web::get().to(get_user_tickets::exec))
    );

    cfg.service(
      web::resource("pre-commit")
      .wrap(Rc::clone(&authn_middleware))
      .route(web::post().to(purchase_pre_commit::exec))
    );

    cfg.service(
      web::resource("event_tickets")
      .wrap(Rc::clone(&authn_middleware))
      .route(web::get().to(tickets_by_type::exec))
    );

    cfg.service(
      web::resource("fiat")
      .wrap(Rc::clone(&authn_middleware))
      .route(web::post().to(save_user_ticket_fiat::exec))
    );

    cfg.service(
      web::resource("fiat/pre-commit")
      .wrap(Rc::clone(&authn_middleware))
      .route(web::post().to(purchase_pre_commit_fiat::exec))
    );

    cfg.service(
      web::resource("{ticket_nft}/verifications")
      .route(web::post().to(verify_ticket::exec))
    );
  }
}
