use std::{
  sync::Arc,
  str::from_utf8,
};
use eyre::{Result, Report};
use actix_multipart::Multipart;
use futures_util::stream::StreamExt;
use ticketland_event_handler::services::path;
use common_data::{
  models::metadata::{Attribute, Metadata},
  helpers::{send_write},
  repositories::event::upsert_event,
};
use crate::{
  utils::store::Store,
};

fn is_supported_media_type(mime_type: mime::Name) -> bool {
  match mime_type {
    mime::IMAGE | mime::PNG | mime::JPEG | mime::GIF | mime::MP4 | mime::MPEG => true,
    _ => false,
  }
}

async fn inspect_moderation_labels(store: Arc<Store>, image_content: Vec<u8>) -> Result<()> {
  let min_confidence = store.config.image_recognition_confidence;
  let labels = store.aws_rekognition.recognise(image_content, Some(min_confidence)).await?;

  // Inspect labels https://docs.aws.amazon.com/rekognition/latest/dg/moderation.html
  // At the moment we dissalow any of the labels to be present. In the future we might relax this
  // and inspect each label and decide accordingly..
  if labels.is_some() && labels.unwrap().len() > 0 {
    return Err(Report::msg("Inappropriate image".to_string()))
  }

  Ok(())
}

pub async fn store_event(
  store: Arc<Store>,
  event_id: String,
  uid: String,
  mut payload: Multipart,
) -> Result<Metadata> {
  let mut metadata = Metadata::default();
  let mut media_content_type = None;
  let mut event_capacity= String::new();

  while let Some(item) = payload.next().await {
    let mut field = item?;

    let mut content = vec![];
    // Field in turn is stream of *Bytes* object
    while let Some(chunk) = field.next().await {
      let chunk = chunk?;
      content.push(chunk);
    }

    let field_name = field.name();
    let mime_type = field.content_type().type_();
    let content = content.concat();
    
    if is_supported_media_type(mime_type) {
      if content.len() > store.config.max_image_size {
        return Err(Report::msg("Image limit".to_string()))
      }

      inspect_moderation_labels(Arc::clone(&store), content.clone()).await?;

      let content_type = field.content_type().subtype();
      media_content_type = Some(content_type.to_string().clone());

      store.minio.upload(
        &path::get_event_file_path(&event_id, &field_name, &content_type.to_string()),
        content.as_ref()
      )
      .await?;
    } else if mime_type.eq(&mime::APPLICATION_OCTET_STREAM.type_()) {
      let value = from_utf8(content.as_ref()).unwrap().to_string();

      match field_name {
        "name" => {
          metadata.name = value;
        },
        "description" => {
          metadata.description = value;
        },
        "event_capacity" => {
          event_capacity = value;
        },
        "trait_type" => {
          metadata.attributes.push(
            serde_json::from_str::<Attribute>(&value)?
          );
        },
        _ => todo!(), // Simply ignore
      }
    }
  };

  if metadata.is_default() && media_content_type.is_none() {
    return Err(Report::msg("Bad request".to_string()))
  }

  // Store the metadata as JSON on S3
  store.minio.upload(
    &path::get_event_metadata_path(&event_id),
    serde_json::to_string(&metadata).unwrap().as_ref(),
  )
  .await?;

  let mut event_map = metadata.to_map();
  // Update the db
  let (query, db_query_params) = upsert_event(
    event_id,
    uid,
    event_capacity,
    media_content_type.unwrap(),
    event_map.remove("location").unwrap(),
    event_map.remove("venue").unwrap(),
    event_map.remove("type").unwrap(),
    event_map.remove("startDate").unwrap(),
    event_map.remove("endDate").unwrap(),
    event_map.remove("category").unwrap(),
    event_map.remove("publicity").unwrap(),
    event_map.remove("paymentType").unwrap(),
    event_map.remove("name").unwrap(),
    event_map.remove("description").unwrap()
  );

  send_write(
    Arc::clone(&store.neo4j),
    query,
    db_query_params,
  ).await?;

  Ok(metadata)
}
