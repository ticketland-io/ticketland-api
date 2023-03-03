use eyre::Result;
use borsh::{BorshSerialize};
use amqp_helpers::producer::retry_producer::RetryProducer;
use ticketland_event_handler::{
  services::path,
  models::event::UploadImageFile,
};

pub struct NewEventQueue {
  image_upload_producer: RetryProducer,
}

impl NewEventQueue {
  pub async fn new(
    rabbitmq_uri: String,
    retry_ttl: u16,
  ) -> Self {
    let image_upload_producer = RetryProducer::new(
      &rabbitmq_uri,
      &"event_image_file",
      &"event_image_file",
      &"event_image_file.new",
      retry_ttl,
      None,
    ).await.unwrap();

    Self {
      image_upload_producer,
    }
  }

  pub async fn new_event(&self, event_id: String, content_type: String) -> Result<()> {
    let img_msg = UploadImageFile {
      event_id: event_id.clone(),
      source_path: path::get_event_file_path(&event_id, "ticket_image", &content_type),
      content_type: content_type.clone(),
    };

    self.image_upload_producer.publish(
      &"event_image_file",
      &"event_image_file.new",
      &img_msg.try_to_vec().unwrap()
    ).await
  }
}
