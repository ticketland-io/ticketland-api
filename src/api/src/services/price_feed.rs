use std::sync::Arc;
use eyre::Result;
use price_feed::actors::price::get_price_key;
use crate::utils::store::Store;

pub async fn get_sui_price(store: Arc<Store>,) -> Result<i64> {
  let mut redis = store.redis_pool.connection().await?;
  let price = redis.get(&get_price_key("sui"))
  .await?
  .parse::<i64>()?;

  Ok(price)
}
