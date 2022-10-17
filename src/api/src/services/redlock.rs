use eyre::{Result, Report};
use redlock_async::{Lock};

pub struct RedLock {
  inner: redlock_async::RedLock,
}

impl RedLock {
  pub async fn new(redis_urls: Vec<&str>) -> Self {
    let inner = redlock_async::RedLock::new(redis_urls);

    Self {inner}
  }

  pub async fn lock(&self, resource: &[u8], ttl: usize) -> Result<Lock> {
    self.inner.lock(resource, ttl)
    .await
    .map_err(|error| Report::msg(format!("{:?}", error)))
  }
}
