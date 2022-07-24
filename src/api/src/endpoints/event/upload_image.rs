use actix_web::{web, HttpResponse, Error};
use actix_multipart::Multipart;
use api_helpers::middleware::auth::AuthData;
use futures_util::stream::StreamExt;
use crate::{
  utils::store::Store,
  services::minio::Minio,
};

pub async fn exec(
  store: web::Data<Store>,
  mut payload: Multipart,
  _auth: AuthData,
) -> Result<HttpResponse, Error> {
  let minio = Minio::new(
    "http://localhost:9000".to_owned(),
    "test-bucket".to_owned(),
    "OSJ90KMK8FNEILHQOKMS".to_owned(),
    "lM02Cnff9RlfQ9cK+tRg5oP3R27glCnPESJ7siW+".to_owned(),
  ).await;

	let mut content = vec![];

  while let Some(item) = payload.next().await {
    let mut field = item?;

    // Field in turn is stream of *Bytes* object
    while let Some(chunk) = field.next().await {
			content.push(chunk?);
    }
  }

	let content =  content.concat();
  minio.upload(content.as_ref()).await.unwrap();

  Ok(HttpResponse::Ok().finish())
}
