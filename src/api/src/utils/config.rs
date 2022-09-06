use std::env;

pub struct Config {
  pub port: u64,
  pub neo4j_host: String,
  pub neo4j_domain: Option<String>,
  pub neo4j_username: String,
  pub neo4j_password: String,
  pub neo4j_database: Option<String>,
  pub firebase_auth_key: String,
  pub cors_origin: Vec<String>,
  pub canva_key: String,
  pub minio_uri: String,
  pub minio_bucket: String,
  pub minio_access_key: String,
  pub minio_secret_key: String,
  pub ipfs_gateway: String,
  pub ipfs_server: String,
  // Rabbitmq envs
  pub rabbitmq_uri: String,
  pub exchange_name: String,
  pub retry_ttl: u16,
  pub rpc_endpoint: String,
  pub ticketland_dapp: String,
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
        cors_origin: env::var("CORS_ORIGIN").unwrap().split(",").map(|val| val.to_owned()).collect(),
        minio_uri: env::var("MINIO_URI").unwrap(),
        minio_bucket: env::var("MINIO_BUCKET").unwrap(),
        minio_access_key: env::var("MINIO_ACCESS_KEY").unwrap(),
        minio_secret_key: env::var("MINIO_SECRET_KEY").unwrap(),
        ipfs_gateway: env::var("IPFS_GATEWAY").unwrap(),
        ipfs_server: env::var("IPFS_SERVER").unwrap(),
        rabbitmq_uri: env::var("RABBITMQ_URI").unwrap(),
        exchange_name: env::var("EXCHANGE_NAME").unwrap(),
        retry_ttl: env::var("RETRY_TTL").unwrap().parse::<u16>().unwrap(),
        rpc_endpoint: env::var("RPC_ENDPOINT").unwrap(),
        canva_key: env::var("CANVA_CLIENT_SECRET").unwrap(),
        ticketland_dapp: env::var("TICKETLAND_DAPP").unwrap(),
      }
    )
  }
}
