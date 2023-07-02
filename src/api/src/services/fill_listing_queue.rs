use eyre::Result;
use borsh::{BorshSerialize};
use amqp_helpers::producer::retry_producer::RetryProducer;
use ticketland_event_handler::{
  models::fill_listing::FillListing,
};

pub struct FillListingQueue {
  producer: RetryProducer,
}

impl FillListingQueue {
  pub async fn new(
    rabbitmq_uri: String,
    retry_ttl: u32,
  ) -> Self {
    let producer = RetryProducer::new(
      &rabbitmq_uri,
      &"fill_listing",
      &"fill_listing",
      &"fill_listing.new",
      retry_ttl,
      None,
    ).await.unwrap();

    Self {
      producer,
    }
  }

  pub async fn new_listing(
    &self,
    buyer_uid: String,
    event_id: String,
    recipient: String,
    seat_index :String,
    listing_sui_address: String,
    txb_bytes: String,
    signature: String,
  ) -> Result<()> {
    let msg = FillListing {
      buyer_uid,
      event_id,
      recipient,
      seat_index,
      listing_sui_address,
      txb_bytes,
      signature,
    };

    self.producer.publish(
      &"fill_listing",
      &"fill_listing.new",
      &msg.try_to_vec().unwrap(),
      true,
    ).await
  }
}
