use eyre::Result;
use redis::{
  aio::Connection,
  cmd,
};

pub struct Redis {
  conn: Connection,
}

impl Redis {
  pub async fn new(redis_url: &str) -> Result<Self> {
    let client = redis::Client::open(redis_url)?;
    let conn = client.get_async_connection().await?;

    Ok(Redis {conn})
  }

  pub async fn set(&mut self, key: &str, value: &str) -> Result<()> {
    cmd("SET")
    .arg(&[key, value])
    .query_async(&mut self.conn).await
    .map_err(Into::<_>::into)
  }
}
