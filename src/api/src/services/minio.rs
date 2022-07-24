use s3::{
  bucket::Bucket,
  BucketConfiguration,
  creds::Credentials,
  region::Region,
  error::S3Error,
};

pub struct Minio {
  bucket: Bucket,
}

impl Minio {
  pub async fn new(
    endpoint: String,
    bucket_name: String,
    access_key: String,
    secret_key: String,
  ) -> Self {
    let region = Region::Custom {
      region: "".into(),
      endpoint: endpoint.into(),
    };

    let credentials = Credentials::new(
      Some(&access_key),
      Some(&secret_key),
      None,
      None,
      None,
    ).expect("Wrong credentials provided");

    let config = BucketConfiguration::default();
    let bucket = Bucket::create_with_path_style(
      &bucket_name,
      region,
      credentials,
      config,
    )
    .await
    .expect("cannot connect to Minio")
    .bucket;

    Self {
      bucket,
    }
  }

  pub async fn upload(&self, content: &[u8]) -> Result<(), S3Error> {
    self.bucket.put_object("/test.png", content).await?;
    
    Ok(())
  }
}
