use std::env;
use solana_sdk::pubkey::Pubkey;
use solana_web3_rust::utils::pubkey_from_str;

pub struct Config {
  pub port: u64,
  pub neo4j_host: String,
  pub neo4j_domain: Option<String>,
  pub neo4j_username: String,
  pub neo4j_password: String,
  pub neo4j_database: Option<String>,
  pub redis_host: String,
  pub redis_password: String,
  pub firebase_auth_key: String,
  pub cors_origin: Vec<String>,
  pub canva_key: String,
  pub stripe_key: String,
  pub stripe_webhook_key: String,
  pub minio_region: String,
  pub minio_bucket: String,
  pub minio_access_key: String,
  pub minio_secret_key: String,
  // Rabbitmq envs
  pub rabbitmq_uri: String,
  pub exchange_name: String,
  pub retry_ttl: u16,
  pub rpc_endpoint: String,
  pub ticketland_api: String,
  pub ticketland_dapp: String,
  pub ticket_nft_program_state: Pubkey,
  pub event_registry_state: Pubkey,
  pub ticket_purchae_protocol_fee: i64,
  pub ticket_verifier_priv_key: String,
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
        redis_host: env::var("REDIS_HOST").unwrap(),
        redis_password: env::var("REDIS_PASSWORD").unwrap(),
        firebase_auth_key: env::var("FIREBASE_API_KEY").unwrap(),
        cors_origin: env::var("CORS_ORIGIN").unwrap().split(",").map(|val| val.to_owned()).collect(),
        minio_region: env::var("MINIO_REGION").unwrap(),
        minio_bucket: env::var("NFT_BUCKET").unwrap(),
        minio_access_key: env::var("MINIO_ACCESS_KEY").unwrap(),
        minio_secret_key: env::var("MINIO_SECRET_KEY").unwrap(),
        rabbitmq_uri: env::var("RABBITMQ_URI").unwrap(),
        exchange_name: env::var("EXCHANGE_NAME").unwrap(),
        retry_ttl: env::var("RETRY_TTL").unwrap().parse::<u16>().unwrap(),
        rpc_endpoint: env::var("RPC_ENDPOINT").unwrap(),
        canva_key: env::var("CANVA_CLIENT_SECRET").unwrap(),
        stripe_key: env::var("STRIPE_CLIENT_SECRET").unwrap(),
        stripe_webhook_key: env::var("STRIPE_WEBHOOK_SECRET").unwrap(),
        ticketland_api: env::var("TICKETLAND_API").unwrap(),
        ticketland_dapp: env::var("TICKETLAND_DAPP").unwrap(),
        ticket_nft_program_state: pubkey_from_str(&env::var("TICKET_NFT_STATE").unwrap()).unwrap(),
        event_registry_state: pubkey_from_str(&env::var("EVENT_REGISTRY_STATE").unwrap()).unwrap(),
        ticket_purchae_protocol_fee: env::var("TICKET_PURCHASE_PROTOCOL_FEE").unwrap().parse::<i64>().unwrap(),
        ticket_verifier_priv_key: env::var("TICKET_VERIFIER_PRIV_KEY").unwrap(),
      }
    )
  }
}
