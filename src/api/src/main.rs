use std::rc::Rc;
use actix_cors::Cors;
use actix_web::{middleware, web, http, App, HttpResponse, HttpServer};
use env_logger::Env;
use std::{env, panic, process};
use api_helpers::{
  middleware::auth::AuthnMiddlewareFactory,
};
use ticketland_api::{
  utils::store::Store,
  endpoints::{
    event::config::config as event_config,
    ticket::config::config as ticket_config,
    listing::config::config as listing_config,
    canva::config::config as canva_config,
    stripe::config::config as stripe_config,
  },
};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
  let orig_hook = panic::take_hook();
  panic::set_hook(Box::new(move |panic_info| {
    orig_hook(panic_info);
    process::exit(1);
  }));

  if env::var("ENV").unwrap() == "development" {
    dotenv::from_filename(".env").expect("cannot load env from a file");
  }

  let store = web::Data::new(Store::new().await);
  let port = store.config.port;
  let firebase_auth_key = store.config.firebase_auth_key.clone();

  env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

  HttpServer::new(move || {
    let authn_middleware = Rc::new(AuthnMiddlewareFactory::new(firebase_auth_key.clone()));

    let cors_origin = store.config.cors_origin.clone();
    let canva_key = store.config.canva_key.clone();

    let cors = Cors::default()
      .allowed_origin_fn(move |origin, _| {
        cors_origin.iter().any(|v| v == origin || v == "*")
      })
      .allowed_methods(vec!["GET", "POST"])
      .allowed_headers(vec![http::header::AUTHORIZATION, http::header::ACCEPT])
      .allowed_header(http::header::CONTENT_TYPE)
      .max_age(3600);

    App::new()
      .app_data(store.clone())
      .wrap(cors)
      .wrap(middleware::Logger::default())
      .service(web::scope("/events").configure(event_config(Rc::clone(&authn_middleware))))
      .service(web::scope("/tickets").configure(ticket_config(Rc::clone(&authn_middleware))))
      .service(web::scope("/listings").configure(listing_config(Rc::clone(&authn_middleware))))
      .service(web::scope("/canva").configure(canva_config(Rc::clone(&authn_middleware), canva_key)))
      .service(web::scope("/stripe").configure(stripe_config(Rc::clone(&authn_middleware))))
      .route("/", web::get().to(|| HttpResponse::Ok()))
  })
  .bind(format!("0.0.0.0:{}", port))?
  .run()
  .await
}
