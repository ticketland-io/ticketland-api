use std::{
  sync::Arc,
  str::from_utf8,
};
use actix_multipart::Multipart;
use futures_util::stream::StreamExt;
use serde::{Serialize, Deserialize};
use ticketland_core::error::Error;
use common_data::{
  helpers::{send_write},
  repositories::event::upsert_event,
};
use crate::utils::store::Store;

#[derive(Serialize, Deserialize)]
struct Attribute {
  trait_type: String,
  value: String,
}

#[derive(Default)]
struct Metadata {
  name: String,
  description: String,
  image: String,
  attributes: Vec<Attribute>,
}

fn is_supported_media_type(mime_type: mime::Name) -> bool {
  match mime_type {
    mime::PNG | mime::JPEG | mime::GIF | mime::MP4 | mime::MPEG => true,
    _ => false,
  }
}

pub async fn store_event(
  store: Arc<Store>,
  event_id: String,
  uid: String,
  mut payload: Multipart,
) -> Result<(), Error> {
  let mut content = vec![];
  let mut metadata = Metadata::default();

  while let Some(item) = payload.next().await {
    let mut field = item
    .map_err(Into::<Error>::into)?;

    // Field in turn is stream of *Bytes* object
    while let Some(chunk) = field.next().await {
      let chunk = chunk.map_err(Into::<Error>::into)?;
      content.push(chunk);
    }

    let field_name = field.name();
    let mime_type = field.content_type().type_();
    let content =  content.concat();

    if is_supported_media_type(mime_type) {    
      store.minio.upload(
        &format!("{}-event_image", event_id),
        content.as_ref()
      )
      .await
      .map_err(Into::<Error>::into)?;
    } else if mime_type.eq(&mime::APPLICATION_OCTET_STREAM.type_()) {
      let value = from_utf8(content.as_ref()).unwrap().to_owned();

      match field_name {
        "name" => {
          metadata.name = value;
        },
        "description" => {
          metadata.description = value;
        },
        "trait_type" => {
          metadata.attributes.push(
            serde_json::from_str(&value)
            .map_err(Into::<Error>::into)?
          );
        },
        _ => todo!(), // Simply ignore
      }
    }
  }

  // 1. Find the Image CID with a dry run on IPFS
  // 2. TODO: store the metadata as JSON on S3

  // Update the db
  let (query, db_query_params) = upsert_event(
    event_id,
    uid,
  );

  send_write(
    Arc::clone(&store.neo4j),
    query,
    db_query_params,
  ).await?;

  Ok(())
}
