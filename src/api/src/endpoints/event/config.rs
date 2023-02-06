use std::{
  rc::Rc,
};
use actix_web::{web};
use api_helpers::{
  middleware::auth::AuthnMiddlewareFactory,
};
use super::{
  create_event,
  get_event_image,
  commit_event,
  get_filtered_events,
  create_event_sales,
  get_event,
  get_events_by_user, 
  get_attended_count
};

pub fn config(authn_middleware: Rc<AuthnMiddlewareFactory>) -> impl FnOnce(&mut web::ServiceConfig) {
  move |cfg: &mut web::ServiceConfig| {
    cfg.service(
      web::resource("")
      .route(web::get().to(get_filtered_events::exec))
    );

    cfg.service(
      web::resource("/current-user")
      .wrap(Rc::clone(&authn_middleware))
      .route(web::get().to(get_events_by_user::exec))
    );

    cfg.service(
      web::resource("{event_id}")
      .wrap(Rc::clone(&authn_middleware))
      .route(web::post().to(create_event::exec))
      .route(web::get().to(get_event::exec))
    );

    cfg.service(
      web::resource("{event_id}/attended-count")
      .wrap(Rc::clone(&authn_middleware))
      .route(web::get().to(get_attended_count::exec))
    );

    cfg.service(
      web::resource("{event_id}/sales")
      .wrap(Rc::clone(&authn_middleware))
      .route(web::post().to(create_event_sales::exec))
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
