use eyre::Result;
use super::price_feed::get_sol_price;

// 1 unit in Stripe is 100
const STRIPE_UNIT: i64 = 100;
const STRIPE_FIXED_FEE: i64 = 30; // 30c
const STRIPE_FEE_PERC: i64 = 29; // 2.9%

/// This is the amount in SOL needed to send a transaction that will mint a new ticket NFT
/// TODO: use the correct value here
const MINT_TICKER_COST_IN_SOL: i64 = 7; // this is 0.007 SOL

fn to_stripe_unit(val: i64) -> i64 {
  val * STRIPE_UNIT
}

pub async fn calculate_price_and_fees(_event_id: &str) -> Result<(i64, i64)> {
  // TODO: We would need to load the Sale account from Solana and then find the sale type of the ticket that is
  // being purchased to find the ticket price. The sale account is a PDA which we calculate using the following seeds.
  //
  // seeds = [
  //  b"sale",
  //  state.key().as_ref(),
  //  sale.ticket_type_index.to_string().as_ref(),
  //  &sale.event_id
  // ]
  // 
  // Note that user might try to pass a sale account for ticket types that are cheap but enter a ticket_nft that belongs to
  // a more expensive ticket type. This won't be possible since a sevice will send the tx to the blockchain using the given ticket_nft
  // which will cause the transaction to faile since there are alreayd checks that avoid something like this to happen.
  // Reading accounts from the chain might be expensive, so we might store this information in our db for faster queries.
  let ticket_price = to_stripe_unit(100);
  
  // This is part of the Event account data. We would need to load the event account from Solana and read this value.
  // Unless we store this information in our database
  let protocol_fee_perc = 100_i64;

  let protocol_fee = (ticket_price * protocol_fee_perc) / 10_000;
  let sol_price = to_stripe_unit(get_sol_price().await?);
  let mint_cost = (MINT_TICKER_COST_IN_SOL * sol_price) / 1000;
  let stripe_fee = (ticket_price * STRIPE_FEE_PERC) / 1000;
  let total_stripe_fees = stripe_fee + STRIPE_FIXED_FEE; // 2.9% + 30c
  let total_fees = protocol_fee + mint_cost + total_stripe_fees;

  Ok((ticket_price as i64, total_fees as i64))
}

pub fn pre_purchase_checks(event_id: &str, ticket_nft: &str, ticket_type_index: u8) -> Result<String> {
  todo!()
}
