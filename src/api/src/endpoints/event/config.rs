use actix_web::{web};
use super::{
  create_event,
  get_event_image,
  confirm_event,
};

pub fn config(cfg: &mut web::ServiceConfig) {
  cfg.service(
    web::resource("{event_id}")
    .route(web::post().to(create_event::exec))
  );

  cfg.service(
    web::resource("{event_id}/confirm")
    .route(web::post().to(confirm_event::exec))
  );
  
  cfg.service(
    web::resource("{event_id}/images")
    .route(web::get().to(get_event_image::exec))
  );
}
