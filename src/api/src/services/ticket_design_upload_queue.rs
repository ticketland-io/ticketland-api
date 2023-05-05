use eyre::Result;
use borsh::{BorshSerialize};
use amqp_helpers::producer::retry_producer::RetryProducer;
use ticketland_event_handler::{
  models::file::UploadFile,
};


pub struct TicketDesignUploadQueue {
  s3_file_upload_producer: RetryProducer,
}

impl TicketDesignUploadQueue {
  pub async fn new(
    rabbitmq_uri: String,
    retry_ttl: u32,
  ) -> Self {
    let s3_file_upload_producer = RetryProducer::new(
      &rabbitmq_uri,
      &"ticket_design_upload",
      &"ticket_design_upload",
      &"ticket_design_upload.new",
      retry_ttl,
      None,
    ).await.unwrap();

    Self {
      s3_file_upload_producer,
    }
  }

  pub async fn new_design(&self, design_id: String, content_type: String, source_url: String) -> Result<()> {
    let file_msg = UploadFile { 
      name: design_id,
      content_type,
      source_url,
    };

    self.s3_file_upload_producer.publish(
      &"ticket_design_upload",
      &"ticket_design_upload.new",
      &file_msg.try_to_vec().unwrap(),
      true,
    ).await
  }
}
