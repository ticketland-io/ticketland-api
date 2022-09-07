use borsh::{BorshSerialize};
use amqp_helpers::producer::retry_producer::RetryProducer;
use ticketland_event_handler::{
  services::path,
  models::event::UploadFile,
};


pub struct NewEventQueue {
  metadata_upload_producer: RetryProducer,
  image_upload_producer: RetryProducer,
}

impl NewEventQueue {
  pub async fn new(
    rabbitmq_uri: String,
    retry_ttl: u16,
  ) -> Self {
    let metadata_upload_producer = RetryProducer::new(
      &rabbitmq_uri,
      &"event_metadata_file",
      &"event_metadata_file",
      &"event_metadata_file.new",
      retry_ttl,
    ).await;

    let image_upload_producer = RetryProducer::new(
      &rabbitmq_uri,
      &"event_image_file",
      &"event_image_file",
      &"event_image_file.new",
      retry_ttl,
    ).await;

    Self {
      metadata_upload_producer,
      image_upload_producer,
    }
  }

  pub async fn on_new_event(&self, event_id: String, content_type: String) {
    let metadata_msg = UploadFile { 
      event_id: event_id.clone(),
      path: path::get_event_file_path(&event_id, &content_type),
    };

    self.metadata_upload_producer.publish(
      &"event_metadata_file",
      &"event_metadata_file.new",
      &metadata_msg.try_to_vec().unwrap()
    ).await;

    let img_msg = UploadFile { 
      event_id: event_id.clone(),
      path: path::get_event_metadata_path(&event_id),
    };

    self.image_upload_producer.publish(
      &"event_image_file",
      &"event_image_file.new",
      &img_msg.try_to_vec().unwrap()
    ).await;
  }
}
