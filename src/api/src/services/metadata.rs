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

#[derive(Serialize, Deserialize, Debug)]
struct Attribute {
  trait_type: String,
  value: String,
}

#[derive(Default, Debug, Serialize)]
struct Metadata {
  name: String,
  description: String,
  image: String,
  attributes: Vec<Attribute>,
}

fn is_supported_media_type(mime_type: mime::Name) -> bool {
  match mime_type {
    mime::IMAGE | mime::PNG | mime::JPEG | mime::GIF | mime::MP4 | mime::MPEG => true,
    _ => false,
  }
}

fn create_ipfs_uri(ipfs_gateway: &str, cid: &str) -> String {
  format!("{}{}", ipfs_gateway, cid)
}

pub async fn store_event(
  store: Arc<Store>,
  event_id: String,
  uid: String,
  mut payload: Multipart,
) -> Result<(), Error> {
  let mut metadata = Metadata::default();

  while let Some(item) = payload.next().await {
    let mut field = item
    .map_err(Into::<Error>::into)?;

    let mut content = vec![];
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
        &format!("{}-event_image.{}", event_id, field.content_type().subtype()),
        content.as_ref()
      )
      .await
      .map_err(Into::<Error>::into)?;


      

      // Find the Image CID with a dry run on IPFS
      let response = store.ipfs.dry_run(content).await?;
      metadata.image = create_ipfs_uri(&store.config.ipfs_gateway, &response.hash);
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
            serde_json::from_str::<Attribute>(&value)
            .map_err(Into::<Error>::into)?
          );
        },
        _ => todo!(), // Simply ignore
      }
    }
  }

  // Store the metadata as JSON on S3
  store.minio.upload(
    &format!("{}-event_metadata.json", event_id),
    serde_json::to_string(&metadata).unwrap().as_ref(),
  )
  .await
  .map_err(Into::<Error>::into)?;

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
