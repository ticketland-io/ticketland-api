use std::sync::Arc;
use ticketland_data::connection_pool::ConnectionPool;
use ticketland_core::{
  services::{
    minio::Minio,
    redis,
    redlock::RedLock,
  },
};
use solana_web3_rust::rpc_client::RpcClient;
use ticketland_ai::image_recognition::aws_rekognition::AwsRekognition;
use super::config::Config;
use crate::{
  services::{
    new_event_queue::NewEventQueue,
    ticket_design_upload_queue::TicketDesignUploadQueue,
    ticket_purchase_queue::TicketPurchaseQueue,
    set_attended_queue::SetAttendedQueue,
    fill_sell_listing_queue::FillSellListingQueue,
  },
};

pub struct Store {
  pub config: Config,
  pub pg_pool: ConnectionPool,
  pub minio: Arc<Minio>,
  pub aws_rekognition: Arc<AwsRekognition>,
  pub redis_pool: redis::ConnectionPool,
  pub redlock: Arc<RedLock>,
  pub rpc_client: Arc<RpcClient>,
  pub new_event_queue: NewEventQueue,
  pub ticket_design_upload_queue: TicketDesignUploadQueue,
  pub ticket_purchase_queue: TicketPurchaseQueue,
  pub fill_sell_listing_queue: FillSellListingQueue,
  pub set_attended_queue: SetAttendedQueue,
}

impl Store {
  pub async fn new() -> Self {
    let config = Config::new().unwrap();

    let pg_pool = ConnectionPool::new(&config.postgres_uri).await;
    let minio = Arc::new(Minio::new(
      None,
      &config.minio_region,
      &config.minio_bucket,
      &config.minio_access_key,
      &config.minio_secret_key,
    ).await);

    let aws_rekognition = Arc::new(AwsRekognition::new(
      &config.aws_rekognition_access_key,
      &config.aws_rekognition_secret_key,
      config.aws_rekognition_region.clone(),
    ).await);

    let redis_pool = redis::ConnectionPool::new(&config.redis_host, &config.redis_password, config.redis_port);
    let redlock = Arc::new(RedLock::new(vec![&config.redis_host], &config.redis_password));

    let new_event_queue = NewEventQueue::new(
      config.rabbitmq_uri.clone(),
      config.retry_ttl,
    ).await;

    let ticket_design_upload_queue = TicketDesignUploadQueue::new(
      config.rabbitmq_uri.clone(),
      config.retry_ttl,
    ).await;

    let ticket_purchase_queue = TicketPurchaseQueue::new(
      config.rabbitmq_uri.clone(),
      config.retry_ttl,
    ).await;

    let fill_sell_listing_queue = FillSellListingQueue::new(
      config.rabbitmq_uri.clone(),
      config.retry_ttl,
    ).await;

    let set_attended_queue = SetAttendedQueue::new(
      config.rabbitmq_uri.clone(),
      config.retry_ttl,
    ).await;

    let rpc_client = Arc::new(RpcClient::new(config.rpc_endpoint.clone(), None));

    Self {
      config,
      pg_pool,
      redis_pool,
      redlock,
      minio,
      aws_rekognition,
      rpc_client,
      new_event_queue,
      ticket_design_upload_queue,
      ticket_purchase_queue,
      fill_sell_listing_queue,
      set_attended_queue,
    }
  }
}
