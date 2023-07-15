use std::{
  sync::Arc,
  str::from_utf8,
};
use chrono::NaiveDateTime;
use eyre::{Result, Report, ContextCompat};
use actix_multipart::Multipart;
use futures_util::stream::StreamExt;
use serde::Deserialize;
use ticketland_event_handler::services::path;
use ticketland_data::{
  models::{
    // properties::{Property, NewProperty},
    event::{Event, Location},
    ticket_type::NewTicketType,
    seat_range::SeatRange, nft_detail::{NewNftDetail, NewEventNftDetail, NewTicketTypeNftDetail}
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

struct TicketTypeNftFile {
  pub ref_name: String,
  pub content_type: String,
  pub arweave_tx_id: String,
}
#[derive(Deserialize)]
struct TicketTypeNft {
  pub name: String,
  pub description: String,
  pub ticket_type_index: i16,
  pub ref_name: String,
}

async fn upload_file(
  store: Arc<Store>,
  event_id: String,
  path: String,
  content: Vec<u8>,
  content_type: String
) -> Result<String> {
  store.minio.upload_with_content_type(
    &path,
    content.as_ref(),
    &content_type,
  )
  .await?;

  let reward_multipler = store.config.arweave_reward_multiplier;

  let tx = store.arweave.upload_data(
    content.into(),
    None,
    reward_multipler,
    None,
    true
  )
  .await
  .map_err(|error|{
    println!("Error uploading file for event {}: {:?}", &event_id, error);
    error
  })?;

  let tx_hash = tx.0.to_string();

  Ok(tx_hash)
}

pub async fn store_event(
  store: Arc<Store>,
  event_id: String,
  uid: String,
  mut payload: Multipart,
) -> Result<()> {
  let mut new_event = Event {
    event_id: event_id.clone(),
    account_id: uid,
    draft: true,
    ..Event::default()
  };
  let mut event_file_arweave_tx_id = String::new();
  let mut cover_media_content_type = None;
  let mut ticket_type_nfts = vec![];
  let mut nft_files = vec![];
  // let mut properties = vec![];

  let mut seat_ranges = vec![];
  let mut ticket_types = vec![];

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
    let path: String;

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
        event_file_arweave_tx_id = upload_file(
          Arc::clone(&store),
          event_id.clone(),
          path::get_event_file_path(&event_id, &field_name),
          content,
          mime_subtype.to_string()
        ).await?;
      } else if field_name.starts_with("nft_file") {
        let fields = field_name.split("nft_file-").collect::<Vec<&str>>();
        let ref_name = fields[1];

        path = path::get_ticket_nft_file_path(&event_id, "nft_file", ref_name);

        let arweave_tx_id = upload_file(
          Arc::clone(&store),
          event_id.clone(),
          path,
          content,
          mime_subtype.to_string()
        ).await?;

        nft_files.push(TicketTypeNftFile {
          ref_name: ref_name.to_string(),
          content_type: mime_subtype.to_string(),
          arweave_tx_id,
        });
      } else {
        return Err(Report::msg("Bad request".to_string()))
      }
    } else if mime_subtype.eq(&mime::APPLICATION_OCTET_STREAM.subtype()) {
      let value = from_utf8(content.as_ref())?.to_string();

      match field_name {
        "name" => {
          new_event.name = value;
        },
        "description" => {
          new_event.description = value;
        },
        "category" => {
          new_event.category = value.parse::<i16>()?;
        },
        "visibility" => {
          new_event.visibility = value.parse::<i16>()?;
        },
        "location" => {
          new_event.location = Some(serde_json::from_str::<Location>(&value)?);
        },
        "venue" => {
          new_event.venue = value;
        },
        "event_type" => {
          new_event.event_type = value.parse::<i16>()?;
        },
        "start_date" => {
          let start_date_ts = value.parse::<i64>()?;
          new_event.start_date = NaiveDateTime::from_timestamp_opt(start_date_ts, 0).context("invalid start_date")?;
        },
        "end_date" => {
          let end_date_ts = value.parse::<i64>()?;
          new_event.end_date = NaiveDateTime::from_timestamp_opt(end_date_ts, 0).context("invalid end_date")?;
        },
        "ticket_type" => {
          ticket_types.push(serde_json::from_str::<NewTicketType>(&value)?);
        },
        "seat_range" => {
          seat_ranges.push(serde_json::from_str::<SeatRange>(&value)?);
        },
        // "property" => {
        //   properties.push(serde_json::from_str::<Property>(&value)?);
        // },
        "ticket_type_nft" => {
          ticket_type_nfts.push(serde_json::from_str::<TicketTypeNft>(&value)?);
        },
        _ => todo!(), // Simply ignore
      }
    }
  };

  if new_event.is_default() && cover_media_content_type.is_none()
  && ticket_types.len() == 0 && ticket_type_nfts.len() != nft_files.len()
  {
    return Err(Report::msg("Bad request".to_string()))
  }

  let nft_event_detail = NewEventNftDetail {
    ref_name: event_id.clone(),
    event_id: event_id.clone(),
    nft_details_id: event_file_arweave_tx_id.clone(),
  };

  let mut nft_details = vec![NewNftDetail {
    nft_name: new_event.name.clone(),
    nft_description: new_event.description.clone(),
    content_type: cover_media_content_type.context("missing content_type")?.clone(),
    arweave_tx_id: event_file_arweave_tx_id.clone(),
  }];
  let mut ticket_type_nfts_details = vec![];
  // let mut properties = vec![];

  nft_files.iter().for_each(|nft_file| {
  let found = ticket_type_nfts
    .iter()
    .find(|ttn| ttn.ref_name == nft_file.ref_name)
    .context("Missing ticket type nft")
    .unwrap();

  nft_details.push(NewNftDetail {
    nft_name: found.name.clone(),
    nft_description: found.description.clone(),
    content_type: nft_file.content_type.clone(),
    arweave_tx_id: nft_file.arweave_tx_id.clone(),
  });
  ticket_type_nfts_details.push(NewTicketTypeNftDetail {
    ref_name: found.ref_name.clone(),
    event_id: event_id.clone(),
    ticket_type_index: found.ticket_type_index,
    nft_details_id: nft_file.arweave_tx_id.clone(),
  });

  // TODO: iterate over properties on body
  // properties.push(NewProperty {
  //   nft_details_id: nft_file.arweave_tx_id.clone(),
  //   trait_type: event_id.clone(),
  //   ticket_type_index: nft_file.ticket_type_index,
  // });
  });

  let mut postgres = store.pg_pool.connection().await?;
  postgres.upsert_event(
    new_event,
    seat_ranges,
    ticket_types,
    nft_details,
    nft_event_detail,
    ticket_type_nfts_details,
    // properties,
  ).await?;

  Ok(())
}

pub async fn update_event_ticket_type_nft_details(
  store: Arc<Store>,
  event_id: String,
  mut payload: Multipart,
) -> Result<()> {
  let mut ticket_type_nfts = vec![];
  let mut nft_files = vec![];
  // let mut properties = vec![];

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
    let path: String;

    if is_supported_media_type(mime_subtype) {
      if content.len() > store.config.max_image_size {
        return Err(Report::msg("Image limit".to_string()))
      }

      // Add pdf moderation logic
      if is_supported_cover_media_type(mime_subtype) {
        inspect_moderation_labels(Arc::clone(&store), content.clone()).await?;
      }

      if field_name.starts_with("nft_file") {
        let fields = field_name.split("nft_file-").collect::<Vec<&str>>();
        let ref_name = fields[1];

        path = path::get_ticket_nft_file_path(&event_id, "nft_file", ref_name);

        let arweave_tx_id = upload_file(
          Arc::clone(&store),
          event_id.clone(),
          path,
          content,
          mime_subtype.to_string()
        ).await?;

        nft_files.push(TicketTypeNftFile {
          ref_name: ref_name.to_string(),
          content_type: mime_subtype.to_string(),
          arweave_tx_id,
        });
      } else {
        return Err(Report::msg("Bad request".to_string()))
      }
    } else if mime_subtype.eq(&mime::APPLICATION_OCTET_STREAM.subtype()) {
      let value = from_utf8(content.as_ref())?.to_string();

      match field_name {
        "ticket_type_nft" => {
          ticket_type_nfts.push(serde_json::from_str::<TicketTypeNft>(&value)?);
        },
        _ => todo!(), // Simply ignore
      }
    }
  };

  if ticket_type_nfts.len() != nft_files.len() {
    return Err(Report::msg("Bad request".to_string()))
  }

  let mut nft_details = vec![];
  let mut ticket_type_nfts_details = vec![];
  // let mut properties = vec![];

  nft_files.iter().for_each(|nft_file| {
  let found = ticket_type_nfts
    .iter()
    .find(|ttn| ttn.ref_name == nft_file.ref_name)
    .context("Missing ticket type nft")
    .unwrap();

  nft_details.push(NewNftDetail {
    nft_name: found.name.clone(),
    nft_description: found.description.clone(),
    content_type: nft_file.content_type.clone(),
    arweave_tx_id: nft_file.arweave_tx_id.clone(),
  });
  ticket_type_nfts_details.push(NewTicketTypeNftDetail {
    ref_name: found.ref_name.clone(),
    event_id: event_id.clone(),
    ticket_type_index: found.ticket_type_index,
    nft_details_id: nft_file.arweave_tx_id.clone(),
  });

  // TODO: iterate over properties on body
  // properties.push(NewProperty {
  //   nft_details_id: nft_file.arweave_tx_id.clone(),
  //   trait_type: event_id.clone(),
  //   ticket_type_index: nft_file.ticket_type_index,
  // });
  });

  let mut postgres = store.pg_pool.connection().await?;
  postgres.update_ticket_type_nft_details(
    nft_details,
    ticket_type_nfts_details,
    // properties,
  ).await?;

  Ok(())
}
