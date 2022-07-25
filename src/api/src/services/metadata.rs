use actix_multipart::Multipart;
use futures_util::stream::StreamExt;
use ticketland_core::error::Error;
use super::minio::Minio;

pub async fn upload_image_to_s3(
  minio: &Minio,
  event_id: &str,
  mut payload: Multipart,
) -> Result<(), Error> {
  let mut content = vec![];

  while let Some(item) = payload.next().await {
    let mut field = item
    .map_err(|_| Error::S3Error)?;


    // Field in turn is stream of *Bytes* object
    while let Some(chunk) = field.next().await {
      let chunk = chunk.map_err(|_| Error::S3Error)?;
      content.push(chunk);
    }
  }

	let content =  content.concat();
  minio.upload(
    &format!("{}-event_image", event_id),
    content.as_ref()
  )
  .await
  .map_err(|_| Error::S3Error)?;

  Ok(())
}
