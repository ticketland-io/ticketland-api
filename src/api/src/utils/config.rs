use std::env;

pub struct Config {
  pub port: u64,
  pub neo4j_host: String,
  pub neo4j_domain: Option<String>,
  pub neo4j_username: String,
  pub neo4j_password: String,
  pub neo4j_database: Option<String>,
  pub firebase_auth_key: String,
  pub cors_origin: String,
  pub minio_uri: String,
  pub minio_bucket: String,
  pub minio_access_key: String,
  pub minio_secret_key: String,
}

impl Config {
  pub fn new() -> Result<Self, env::VarError> {
    Result::Ok(
      Self {
        port: env::var("PORT").unwrap().parse::<u64>().unwrap(),
        neo4j_host: env::var("NEO4J_HOST").unwrap(),
        neo4j_domain: None,
        neo4j_username: env::var("NEO4J_USERNAME").unwrap(),
        neo4j_password: env::var("NEO4J_PASSWORD").unwrap(),
        neo4j_database: env::var("NEO4J_DATABASE").ok(),
        firebase_auth_key: env::var("FIREBASE_API_KEY").unwrap(),
        cors_origin: env::var("CORS_ORIGIN").unwrap(),
        minio_uri: env::var("MINIO_URI").unwrap(),
        minio_bucket: env::var("MINIO_BUCKET").unwrap(),
        minio_access_key: env::var("MINIO_ACCESS_KEY").unwrap(),
        minio_secret_key: env::var("MINIO_SECRET_KEY").unwrap(),
      }
    )
  }
}
