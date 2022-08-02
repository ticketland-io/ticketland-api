use actix_web::{web};
use super::{
  create_event,
  get_event_image,
  commit_event,
};

pub fn config(cfg: &mut web::ServiceConfig) {
  cfg.service(
    web::resource("{event_id}")
    .route(web::post().to(create_event::exec))
  );

  cfg.service(
    web::resource("{event_id}/commits")
    .route(web::post().to(commit_event::exec))
  );
  
  cfg.service(
    web::resource("{event_id}/images")
    .route(web::get().to(get_event_image::exec))
  );
}
