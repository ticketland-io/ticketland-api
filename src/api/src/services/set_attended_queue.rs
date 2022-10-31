use eyre::Result;
use borsh::{BorshSerialize};
use amqp_helpers::producer::retry_producer::RetryProducer;
use ticketland_event_handler::{
  models::metadata::SetAttended,
};

pub struct SetAttendedQueue {
  set_attended_producer: RetryProducer,
}

impl SetAttendedQueue {
  pub async fn new(
    rabbitmq_uri: String,
    retry_ttl: u16,
  ) -> Self {
    let set_attended_producer = RetryProducer::new(
      &rabbitmq_uri,
      &"set_attended",
      &"set_attended",
      &"set_attended.new",
      retry_ttl,
    ).await.unwrap();

    Self {
      set_attended_producer,
    }
  }

  pub async fn on_set_attended(&self, event_id: String, ticket_nft: String) -> Result<()> {
    let set_attended_msg = SetAttended {
      event_id: event_id.clone(),
      ticket_nft: ticket_nft.clone(),
    };

    self.set_attended_producer.publish(
      &"set_attended",
      &"set_attended.new",
      &set_attended_msg.try_to_vec().unwrap()
    ).await
  }
}
