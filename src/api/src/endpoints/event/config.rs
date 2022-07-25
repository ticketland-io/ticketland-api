use actix_web::{web};
use super::{
  create_event,
  read_event,
};

pub fn config(cfg: &mut web::ServiceConfig) {
  cfg.service(
    web::resource("{event_id}")
    .route(web::post().to(create_event::exec))
    .route(web::get().to(read_event::exec))
  );
}
