use ticketland_core::error::Error;

// TODO: we need to read the SOL price from Redis.
// There will be a Price Oracle service that will be updating this entry
// on a regular basis
pub async fn get_sol_price() -> Result<i64, Error> {
  Ok(30)
}
