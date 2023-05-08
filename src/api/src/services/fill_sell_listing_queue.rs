use eyre::Result;
use borsh::{BorshSerialize};
use amqp_helpers::producer::retry_producer::RetryProducer;
use ticketland_event_handler::{
  models::fill_sell_listing::FillSellListing,
};

pub struct FillSellListingQueue {
  producer: RetryProducer,
}

impl FillSellListingQueue {
  pub async fn new(
    rabbitmq_uri: String,
    retry_ttl: u32,
  ) -> Self {
    let producer = RetryProducer::new(
      &rabbitmq_uri,
      &"fill_sell_listing",
      &"fill_sell_listing",
      &"fill_sell_listing.new",
      retry_ttl,
      None,
    ).await.unwrap();

    Self {
      producer,
    }
  }

  pub async fn new_sell_listing(
    &self,
    buyer_uid: String,
    event_id: String,
    sale_account: String,
    ticket_nft: String,
    recipient: String,
    sell_listing_account: String,
  ) -> Result<()> {
    let msg = FillSellListing {
      buyer_uid,
      event_id,
      sale_account,
      ticket_nft,
      recipient,
      sell_listing_account,
    };

    self.producer.publish(
      &"fill_sell_listing",
      &"fill_sell_listing.new",
      &msg.try_to_vec().unwrap(),
      true,
    ).await
  }
}
