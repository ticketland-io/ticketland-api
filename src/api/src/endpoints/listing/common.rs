use serde::{Deserialize};

#[derive(Deserialize)]
pub struct ListingParams {
  pub listing_account: String,
}
