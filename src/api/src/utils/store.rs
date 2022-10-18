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
use super::config::Config;
use crate::{
  services::new_event_queue::NewEventQueue,
  services::ticket_design_upload_queue::TicketDesignUploadQueue,
  services::ticket_purchase_queue::TicketPurchaseQueue,
};

pub struct Store {
  pub config: Config,
  pub neo4j: Arc<Addr<Neo4jActor>>,
  pub minio: Arc<Minio>,
  pub redis: Arc<Mutex<Redis>>,
  pub redlock: Arc<RedLock>,
  pub rpc_client: Arc<RpcClient>,
  pub new_event_queue: NewEventQueue,
  pub ticket_design_upload_queue: TicketDesignUploadQueue,
  pub ticket_purchase_queue: TicketPurchaseQueue,
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

    let rpc_client = Arc::new(RpcClient::new(config.rpc_endpoint.clone(), None));

    Self {
      config,
      neo4j,
      redis,
      redlock,
      minio,
      rpc_client,
      new_event_queue,
      ticket_design_upload_queue,
      ticket_purchase_queue,
    }
  }
}
