use std::{
  future::{ready, Ready as StdReady},
  rc::Rc,
  str,
};
use actix_web::{
  HttpMessage,
  web::{self},
  dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
  Error,
  error::ErrorUnauthorized,
};
use sha2::Sha256;
use hmac::{Hmac, Mac};
use chrono::{Utc};
use futures_util::{
  stream::StreamExt,
  future::{LocalBoxFuture},
};

const LENIENCY_IN_SECS: i64 = 300;
const VERSION: &str = "v1";

const PATHS: [&str; 2] = [
    "/canva/configuration",
    "/canva/publish/resources/upload",
  ];

pub struct CanvaMiddlewareFactory {
  canva_key: String,
}

impl CanvaMiddlewareFactory {
  pub fn new(canva_key: String) -> Self {
    Self {canva_key}
  }
}

pub struct CanvaMiddleware<S> {
  service: Rc<S>,
  canva_key: String,
}

impl<S, B> Transform<S, ServiceRequest> for CanvaMiddlewareFactory
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = CanvaMiddleware<S>;
    type Future = StdReady<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
      ready(Ok(CanvaMiddleware {
        service: Rc::new(service),
        canva_key: self.canva_key.clone(),
      }))
    }
}

impl<S, B> CanvaMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static 
{
  fn calculate_sig(canva_key: String, message: String) -> String {
    let key = base64::decode(canva_key).unwrap();
    let mut mac = Hmac::<Sha256>::new_from_slice(key.as_ref()).unwrap();
    
    mac.update(message.as_bytes());
    hex::encode(&mac.finalize().into_bytes()[..])
  }
}

impl<S, B> Service<ServiceRequest> for CanvaMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static
{
  type Response = ServiceResponse<B>;
  type Error = Error;
  type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

  forward_ready!(service);

  fn call(&self, mut req: ServiceRequest) -> Self::Future {
    let srv = self.service.clone();
    let canva_key = self.canva_key.clone();
    
    Box::pin(
      async move {
        let ts;

        {
          let headers = req.headers();
          let x_canva_timestamp = headers.get("X-Canva-Timestamp")
            .map(|val| val.to_str());

          if x_canva_timestamp.is_none() {
            return Err(ErrorUnauthorized("Unauthorized"))
          }
  
          let x_canva_timestamp  = x_canva_timestamp.unwrap();
          if x_canva_timestamp.is_err() {
            return Err(ErrorUnauthorized("Unauthorized"))
          }

          ts = x_canva_timestamp.unwrap().to_string();

          if Utc::now().timestamp() - ts.parse::<i64>().unwrap() > LENIENCY_IN_SECS {
            println!("ts");
            return Err(ErrorUnauthorized("Unauthorized"))
          }
        }

        let path = PATHS.into_iter().find(|p| *p == req.path());
        if path.is_none() {
          println!("path");
          return Err(ErrorUnauthorized("Unauthorized"))
        }

        let mut body;
        let mut raw_body;

        {
          body = req.take_payload();
          raw_body = web::BytesMut::new();

          while let Some(item) = body.next().await {
            raw_body.extend_from_slice(&item?);
          }
        }

        {
          let headers = req.headers();
          let signatures = headers.get("X-Canva-Signatures")
            .map(|val| val.to_str());
          
          if signatures.is_none() {
            println!("signatures none");
            return Err(ErrorUnauthorized("Unauthorized"))
          }

          let signatures  = signatures.unwrap();
          if signatures.is_err() {
            println!("signatures str");
            return Err(ErrorUnauthorized("Unauthorized"))
          }

          let signatures: Vec<&str> = signatures.unwrap().split(",").collect();
          println!("path {:?}", path.unwrap().replace("/canva", ""));
          let message = format!("{}:{}:{}:{:?}", VERSION, ts, path.unwrap().replace("/canva", ""), raw_body);
          let sig = Self::calculate_sig(canva_key, message);
          
          println!("Sig >>>>>>>> {:?}", sig);
          println!("Signatures >>>>>>>> {:?}", signatures);

          if !signatures.iter().any(|v| *v == sig) {
            println!("signatures not found");
            return Err(ErrorUnauthorized("Unauthorized"))
          }
        }

        return Ok(srv.call(req).await?)
      }
    )
  }
}
