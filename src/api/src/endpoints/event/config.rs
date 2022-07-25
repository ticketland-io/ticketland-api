use actix_web::{web};
use super::{
  upload_image,
  create_event,
  read_image,
};

pub fn config(cfg: &mut web::ServiceConfig) {
  cfg.service(
    web::resource("/images")
    .route(web::post().to(upload_image::exec))
    .route(web::get().to(read_image::exec))
  );

  cfg.service(
    web::resource("/events")
    .route(web::post().to(create_event::exec))
  );
}
