use std::{
  sync::Arc,
  str::from_utf8,
};
use chrono::NaiveDateTime;
use eyre::{Result, Report, ContextCompat};
use actix_multipart::Multipart;
use futures_util::stream::StreamExt;
use ticketland_event_handler::services::path;
use ticketland_data::{
  models::{
    metadata::{Attribute, Metadata},
    event::{Event, Location, TicketImage},
  },
};
use crate::{
  utils::store::Store,
};

fn is_supported_media_type(mime_type: mime::Name) -> bool {
  match mime_type {
    mime::PDF | mime::PNG | mime::JPEG | mime::GIF | mime::MP4 | mime::MPEG => true,
    _ => false,
  }
}

fn is_supported_cover_media_type(mime_type: mime::Name) -> bool {
  match mime_type {
    mime::PNG | mime::JPEG | mime::GIF | mime::MP4 | mime::MPEG => true,
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
  let mut cover_media_content_type = None;
  let mut ticket_images = vec![];
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
    let mime_subtype = field.content_type().subtype();
    let content = content.concat();

    if is_supported_media_type(mime_subtype) {
      if content.len() > store.config.max_image_size {
        return Err(Report::msg("Image limit".to_string()))
      }
      
      // Add pdf moderation logic
      if is_supported_cover_media_type(mime_subtype) {
        inspect_moderation_labels(Arc::clone(&store), content.clone()).await?;
      }

      if field_name == "cover_image" {
        cover_media_content_type = Some(mime_subtype.to_string().clone());
      } else if field_name.contains("ticket_image") {
        let ticket_image_type = field_name[field_name.len() - 1..].parse()?;
        ticket_images.push(TicketImage {
          event_id: event_id.clone(),
          ticket_image_type,
          content_type: mime_subtype.to_string().clone(),
          arweave_tx_id: None,
          uploaded: false,
        });
      } else {
        return Err(Report::msg("Bad request".to_string()))
      }

      store.minio.upload_with_content_type(
        &path::get_event_file_path(&event_id, &field_name),
        content.as_ref(),
        &mime_subtype.to_string(),
      )
      .await?;
    } else if mime_subtype.eq(&mime::APPLICATION_OCTET_STREAM.subtype()) {
      let value = from_utf8(content.as_ref())?.to_string();

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

  if metadata.is_default() && cover_media_content_type.is_none() && ticket_images.len() == 0 {
    return Err(Report::msg("Bad request".to_string()))
  }

  // Store the metadata as JSON on S3
  store.minio.upload(
    &path::get_event_metadata_path(&event_id),
    serde_json::to_string(&metadata)?.as_ref(),
  )
  .await?;

  let mut event_map = metadata.to_map();

  let start_date = event_map.remove("startDate").context("missing startDate")?.parse::<i64>()?;
  let start_date = NaiveDateTime::from_timestamp_opt(start_date, 0).context("invalid start_date")?;

  let end_date = event_map.remove("endDate").context("missing endDate")?.parse::<i64>()?;
  let end_date = NaiveDateTime::from_timestamp_opt(end_date, 0).context("invalid start_date")?;

  let location = if let Some(location) = event_map.remove("location") {
    Some(serde_json::from_str::<Location>(&location)?)
  } else {
    None
  };

  let mut postgres = store.pg_pool.connection().await?;
  postgres.upsert_event(Event {
    event_id,
    account_id: uid,
    created_at: None,
    name: event_map.remove("name").context("missing name")?,
    description: event_map.remove("description").context("missing description")?,
    location,
    venue: Some(event_map.remove("venue").context("missing venue")?),
    event_type: event_map.remove("type").context("missing type")?.parse()?,
    visibility: event_map.remove("visibility").context("missing visibility")?.parse()?,
    start_date,
    end_date,
    category: event_map.remove("category").context("missing category")?.parse()?,
    event_capacity,
    arweave_tx_id: None,
    webbundle_arweave_tx_id: None,
    draft: true,
  },
  ticket_images,
  ).await?;

  Ok(metadata)
}
