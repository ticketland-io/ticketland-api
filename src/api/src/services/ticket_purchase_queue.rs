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
    retry_ttl: u16,
  ) -> Self {
    let producer = RetryProducer::new(
      &rabbitmq_uri,
      &"ticket_purchase",
      &"ticket_purchase",
      &"ticket_purchase.new",
      retry_ttl,
    ).await;

    Self {
      producer,
    }
  }

  pub async fn on_new_ticket_purcchase(
    &self,
    event_account: String,
    sale_account: String,
    ticket_nft: String,
    recipient: String,
    seat_index: String,
    seat_name: String,
  ) {
    let msg = TicketPurchase { 
      event_account,
      sale_account,
      ticket_nft,
      recipient,
      seat_index,
      seat_name,
    };

    self.producer.publish(
      &"ticket_purchase",
      &"ticket_purchase.new",
      &msg.try_to_vec().unwrap()
    ).await;
  }
}
