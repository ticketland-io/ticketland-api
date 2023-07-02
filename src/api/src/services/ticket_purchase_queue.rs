use eyre::Result;
use borsh::{BorshSerialize};
use amqp_helpers::producer::retry_producer::RetryProducer;
use ticketland_event_handler::{
  models::ticket_purchase::TicketPurchase,
};

pub struct TicketPurchaseQueue {
  producer: RetryProducer,
}

impl TicketPurchaseQueue {
  pub async fn new(
    rabbitmq_uri: String,
    retry_ttl: u32,
  ) -> Self {
    let producer = RetryProducer::new(
      &rabbitmq_uri,
      &"ticket_purchase",
      &"ticket_purchase",
      &"ticket_purchase.new",
      retry_ttl,
      None,
    ).await.unwrap();

    Self {
      producer,
    }
  }

  pub async fn new_ticket_purchase(
    &self,
    buyer_uid: String,
    event_id: String,
    recipient: String,
    seat_index: String,
    seat_name: String,
    txb_bytes: String,
    signature: String,
  ) -> Result<()> {
    let msg = TicketPurchase {
      buyer_uid,
      event_id,
      recipient,
      seat_index,
      seat_name,
      txb_bytes,
      signature,
    };

    self.producer.publish(
      &"ticket_purchase",
      &"ticket_purchase.new",
      &msg.try_to_vec().unwrap(),
      true,
    ).await
  }
}
