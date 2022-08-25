use std::{
  rc::Rc,
};
use actix_web::{web};
use api_helpers::{
  middleware::auth::AuthnMiddlewareFactory,
};
use super::{
  get_all_events,
  create_event,
  get_event_image,
  commit_event,
};

pub fn config(authn_middleware: Rc<AuthnMiddlewareFactory>) -> impl FnOnce(&mut web::ServiceConfig) {
  move |cfg: &mut web::ServiceConfig| {
    cfg.service(
      web::resource("")
      .route(web::get().to(get_all_events::exec))
    );

    cfg.service(
      web::resource("/current-user")
      .wrap(Rc::clone(&authn_middleware))
      .route(web::get().to(get_all_events::exec))
    );

    cfg.service(
      web::resource("{event_id}")
      .wrap(Rc::clone(&authn_middleware))
      .route(web::post().to(create_event::exec))
    );

    cfg.service(
      web::resource("{event_id}/commits")
      .wrap(Rc::clone(&authn_middleware))
      .route(web::post().to(commit_event::exec))
    );
    
    cfg.service(
      web::resource("{event_id}/images")
      .wrap(Rc::clone(&authn_middleware))
      .route(web::get().to(get_event_image::exec))
    );
  }
}
