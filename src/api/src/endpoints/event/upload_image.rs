use actix_web::{web, HttpResponse, Error};
use actix_multipart::Multipart;
use api_helpers::middleware::auth::AuthData;
use futures_util::stream::StreamExt;
use crate::{
  utils::store::Store,
};

pub async fn exec(
  store: web::Data<Store>,
  mut payload: Multipart,
  _auth: AuthData,
) -> Result<HttpResponse, Error> {

	let mut content = vec![];

  while let Some(item) = payload.next().await {
    let mut field = item?;

    // Field in turn is stream of *Bytes* object
    while let Some(chunk) = field.next().await {
			content.push(chunk?);
    }
  }

	let content =  content.concat();
  store.minio.upload(content.as_ref()).await.unwrap();

  Ok(HttpResponse::Ok().finish())
}
