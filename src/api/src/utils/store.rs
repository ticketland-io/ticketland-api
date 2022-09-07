use std::sync::Arc;
use actix::prelude::*;
use ticketland_core::{
  actor::neo4j::Neo4jActor,
  services::{
    minio::Minio,
    ipfs::Ipfs,
  },
};
use solana_client::nonblocking::rpc_client::RpcClient;
use super::config::Config;
use crate::{
  services::new_event_queue::NewEventQueue,
  services::ticket_design_upload_queue::TicketDesignUploadQueue,
};

pub struct Store {
  pub config: Config,
  pub neo4j: Arc<Addr<Neo4jActor>>,
  pub minio: Arc<Minio>,
  pub rpc_client: Arc<RpcClient>,
  pub ipfs: Ipfs,
  pub new_event_queue: NewEventQueue,
  pub ticket_design_upload_queue: TicketDesignUploadQueue,
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
      &config.minio_uri,
      &config.minio_bucket,
      &config.minio_access_key,
      &config.minio_secret_key,
    ).await);

    let ipfs = Ipfs::new(config.ipfs_server.clone());

    let new_event_queue = NewEventQueue::new(
      config.rabbitmq_uri.clone(),
      config.retry_ttl,
    ).await;

    let ticket_design_upload_queue = TicketDesignUploadQueue::new(
      config.rabbitmq_uri.clone(),
      config.retry_ttl,
    ).await;

    let rpc_client = Arc::new(RpcClient::new(config.rpc_endpoint.clone()));

    Self {
      config,
      neo4j,
      minio,
      rpc_client,
      ipfs,
      new_event_queue,
      ticket_design_upload_queue,
    }
  }
}
