use serde::{Deserialize};

#[derive(Deserialize)]
pub struct ListingParams {
  pub listing_id: String,
}

#[derive(Deserialize)]
pub struct OfferParams {
  pub offer_id: String,
}
