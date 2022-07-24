use actix_web::{web};
use super::{
  upload_image,
};

pub fn config(cfg: &mut web::ServiceConfig) {
  cfg.service(
    web::resource("")
    .route(web::post().to(upload_image::exec))
  );
}
