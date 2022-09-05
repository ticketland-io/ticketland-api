use std::{
  rc::Rc,
};
use actix_web::{web};
use api_helpers::{
  middleware::auth::AuthnMiddlewareFactory,
};
use super::{
  publish,
  create_canva_user,
  canva_configuration,
};

pub fn config(authn_middleware: Rc<AuthnMiddlewareFactory>) -> impl FnOnce(&mut web::ServiceConfig) {
  move |cfg: &mut web::ServiceConfig| {
    cfg.service(
      web::resource("users")
      .wrap(Rc::clone(&authn_middleware))
      .route(web::post().to(create_canva_user::exec))
    );

    // This is the endpoint Canva will use to check if a user is authenticated
    cfg.service(
      web::resource("configuration")
      .route(web::post().to(canva_configuration::exec))
    );

    // This is the endpoint Canva will upload the files tos
    cfg.service(
      web::resource("publish/resources/upload")
      .route(web::post().to(publish::exec))
    );
  }
}
