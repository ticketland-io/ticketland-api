use serde::{Deserialize};

#[derive(Deserialize)]
pub struct ListingParams {
  pub event_id: String,
  pub listing_account: String,
}
