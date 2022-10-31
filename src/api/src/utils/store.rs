use std::sync::{Arc, Mutex};
use actix::prelude::*;
use ticketland_core::{
  actor::neo4j::Neo4jActor,
  services::{
    minio::Minio,
    redis::Redis,
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
    fill_sell_listing_queue::FillSellListingQueue,
  },
};

pub struct Store {
  pub config: Config,
  pub neo4j: Arc<Addr<Neo4jActor>>,
  pub minio: Arc<Minio>,
  pub aws_rekognition: Arc<AwsRekognition>,
  pub redis: Arc<Mutex<Redis>>,
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

    let neo4j = Arc::new(
      Neo4jActor::new(
        config.neo4j_host.clone(),
        config.neo4j_domain.clone(),
        config.neo4j_username.clone(),
        config.neo4j_password.clone(),
        config.neo4j_database.clone(),
      )
      .await
      .start(),
    );

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

    let redis = Arc::new(Mutex::new(Redis::new(&config.redis_host, &config.redis_password).await.unwrap()));
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
      neo4j,
      redis,
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
