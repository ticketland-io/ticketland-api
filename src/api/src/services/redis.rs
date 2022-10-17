use eyre::Result;
use redis::{
  aio::Connection,
  cmd,
};

pub struct Redis {
  conn: Connection,
}

impl Redis {
  pub async fn new(redis_host: &str, password: &str) -> Result<Self> {
    let conn_string = format!("redis://:{}@{}:6379", password, redis_host);
    let client = redis::Client::open(conn_string)?;
    let conn = client.get_async_connection().await?;

    Ok(Redis {conn})
  }

  pub async fn set(&mut self, key: &str, value: &str) -> Result<()> {
    cmd("SET")
    .arg(&[key, value])
    .query_async(&mut self.conn).await
    .map_err(Into::<_>::into)
  }

  pub async fn get(&mut self, key: &str) -> Result<()> {
    cmd("GET")
    .arg(&[key])
    .query_async(&mut self.conn).await
    .map_err(Into::<_>::into)
  }

  pub async fn mget(&mut self, keys: &[&str]) -> Result<()> {
    cmd("MGET")
    .arg(keys)
    .query_async(&mut self.conn).await
    .map_err(Into::<_>::into)
  }

  pub async fn keys(&mut self, key_pattern: &str) -> Result<()> {
    cmd("keys")
    .arg(key_pattern)
    .query_async(&mut self.conn).await
    .map_err(Into::<_>::into)
  }
}
