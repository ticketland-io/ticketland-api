use std::{
  rc::Rc,
};
use actix_web::{web};
use api_helpers::{
  middleware::{
    auth::AuthnMiddlewareFactory,
    canva::CanvaMiddlewareFactory,
  },
};
use super::{
  publish,
  create_canva_user,
  canva_configuration,
  auth,
};

pub fn config(authn_middleware: Rc<AuthnMiddlewareFactory>, canva_key: String) -> impl FnOnce(&mut web::ServiceConfig) {
  let canva_middleware_factory = Rc::new(CanvaMiddlewareFactory::new(canva_key));

  move |cfg: &mut web::ServiceConfig| {
    cfg.service(
      web::resource("users")
      .wrap(Rc::clone(&authn_middleware))
      .route(web::post().to(create_canva_user::exec))
    );

    // This is the endpoint Canva will use to check if a user is authenticated
    cfg.service(
      web::resource("configuration")
      .wrap(Rc::clone(&canva_middleware_factory))
      .route(web::post().to(canva_configuration::exec))
    );

    // This is the endpoint Canva will upload the files tos
    cfg.service(
      web::resource("publish/resources/upload")
      .wrap(Rc::clone(&canva_middleware_factory))
      .route(web::post().to(publish::exec))
    );
    
    cfg.service(
      web::resource("auth")
      .wrap(Rc::clone(&canva_middleware_factory))
      .route(web::get().to(auth::exec))
    );
  }
}
