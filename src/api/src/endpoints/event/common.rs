use serde::{Deserialize};

#[derive(Deserialize)]
pub struct EventParams {
  pub event_id: String,
}
