use std::{
  rc::Rc,
};
use actix_web::{web};
use api_helpers::{
  middleware::auth::AuthnMiddlewareFactory,
};
use super::{
  publish,
};

pub fn config(_authn_middleware: Rc<AuthnMiddlewareFactory>) -> impl FnOnce(&mut web::ServiceConfig) {
  move |cfg: &mut web::ServiceConfig| {
    cfg.service(
      // This is the endpoint Canva will upload the files tos
      web::resource("publish/resources/upload")
      .route(web::post().to(publish::exec))
    );
  }
}
