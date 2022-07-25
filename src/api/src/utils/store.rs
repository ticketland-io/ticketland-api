use std::sync::Arc;
use actix::prelude::*;
use ticketland_core::actor::{neo4j::Neo4jActor};
use super::config::Config;
use crate::services::minio::Minio;

pub struct Store {
  pub config: Config,
  pub neo4j: Arc<Addr<Neo4jActor>>,
  pub minio: Minio,
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

    let minio = Minio::new(
      &config.minio_uri,
      &config.minio_bucket,
      &config.minio_access_key,
      &config.minio_secret_key,
    ).await;

    
    Self {
      config,
      neo4j,
      minio,
    }
  }
}
