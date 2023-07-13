use std::{
  rc::Rc,
};
use actix_web::{web};
use api_helpers::{
  middleware::auth::AuthnMiddlewareFactory,
};
use super::{
  save_user_cnt,
  get_event_cnts,
  get_user_cnts,
  verify_cnt,
  purchase_pre_commit,
  save_user_cnt_fiat,
  purchase_pre_commit_fiat,
  get_cnt_info,
  claim_nfts,
};

pub fn config(authn_middleware: Rc<AuthnMiddlewareFactory>) -> impl FnOnce(&mut web::ServiceConfig) {
  move |cfg: &mut web::ServiceConfig| {
    cfg.service(
      web::resource("")
     
      .route(web::post().to(save_user_cnt::exec).wrap(Rc::clone(&authn_middleware)))
      .route(web::get().to(get_event_cnts::exec))
    );
    cfg.service(
      web::resource("current-user")
      .wrap(Rc::clone(&authn_middleware))
      .route(web::get().to(get_user_cnts::exec))
    );

    cfg.service(
      web::resource("pre-commit")
      .wrap(Rc::clone(&authn_middleware))
      .route(web::post().to(purchase_pre_commit::exec))
    );

    cfg.service(
      web::resource("fiat")
      .wrap(Rc::clone(&authn_middleware))
      .route(web::post().to(save_user_cnt_fiat::exec))
    );

    cfg.service(
      web::resource("fiat/pre-commit")
      .wrap(Rc::clone(&authn_middleware))
      .route(web::post().to(purchase_pre_commit_fiat::exec))
    );

    cfg.service(
      web::resource("nft-claims")
      .wrap(Rc::clone(&authn_middleware))
      .route(web::post().to(claim_nfts::exec))
    );

    cfg.service(
      web::resource("{cnt_sui_address}/verifications")
      .route(web::post().to(verify_cnt::exec))
    );

    cfg.service(
      web::resource("{cnt_sui_address}")
      .wrap(Rc::clone(&authn_middleware))
      .route(web::get().to(get_cnt_info::exec))
    );
  }
}
