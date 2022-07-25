use std::sync::Arc;
use actix_multipart::Multipart;
use futures_util::stream::StreamExt;
use ticketland_core::error::Error;
use common_data::{
  helpers::{send_write},
  repositories::event::upsert_event,
};
use crate::utils::store::Store;

pub async fn store_new_tmp_image(
  store: Arc<Store>,
  event_id: String,
  uid: String,
  mut payload: Multipart,
) -> Result<(), Error> {
  let mut content = vec![];
  // let mut content_type = "";
  while let Some(item) = payload.next().await {
    let mut field = item
    .map_err(Into::<Error>::into)?;

    // content_type = 

    println!("{:?}", field.content_type());

    // Field in turn is stream of *Bytes* object
    while let Some(chunk) = field.next().await {
      let chunk = chunk.map_err(Into::<Error>::into)?;
      content.push(chunk);
      println!("{:?}", content);
    }
  }

	let content =  content.concat();
  
  store.minio.upload(
    &format!("{}-event_image", event_id),
    content.as_ref()
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
