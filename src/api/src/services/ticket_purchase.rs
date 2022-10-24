use std::{
  sync::Arc,
  str::FromStr,
};
use eyre::{Result, Report};
use program_artifacts::{
  ticket_sale::account_data::{Sale, SaleType},
  ticket_nft::pda,
};
use solana_sdk::{
  pubkey::Pubkey,
  commitment_config::CommitmentConfig,
};
use crate::utils::store::Store;

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

pub async fn calculate_price_and_fees(
  ticket_price: i64,
  protocol_fee_perc: i64
) -> Result<(i64, i64)> {
  let protocol_fee = (ticket_price * protocol_fee_perc) / 10_000;
  let sol_price = to_stripe_unit(get_sol_price().await?);
  let mint_cost = (MINT_TICKER_COST_IN_SOL * sol_price) / 1000;
  let stripe_fee = (ticket_price * STRIPE_FEE_PERC) / 1000;
  let total_stripe_fees = stripe_fee + STRIPE_FIXED_FEE; // 2.9% + 30c
  let total_fees = protocol_fee + mint_cost + total_stripe_fees;

  Ok((ticket_price as i64, total_fees as i64))
}

pub async fn pre_purchase_checks(
  store: Arc<Store>,
  ticket_nft_program_state: &Pubkey,
  seat_index: u32,
  sale_account: &str,
  ticket_nft: &str
) -> Result<(i64, i64)> {
  // TODO: Reading accounts from the chain might be expensive, so we might store this information in our db for faster queries.
  let sale = store.rpc_client.get_anchor_account_data::<Sale>(&Pubkey::from_str(&sale_account)?).await?;

  let (ticket_nft_pda, _) = pda::ticket_nft(
    ticket_nft_program_state,
    seat_index,
    std::str::from_utf8(&sale.event_id)?,
    sale.ticket_type_index,
  );

  // Using PDA seeds allows us to impose some constraints and do some validation.
  // Ticket nfts are PDAs and part of the seed list is the ticket type index. This allows
  // us to validatate that user does not pass a ticket type which has has lower price but 
  // use a ticket nft that is of a higher, more expensive type.
  if ticket_nft_pda.to_string() != ticket_nft {
    return Err(Report::msg("Invalid ticket_nft"))?
  }

  // We need to check whether this ticket nft account exists. If it does it means that someone else
  // has already purchased it. We could alternatively load the event_capacity account and check the
  // bit array for availability.
  let is_ticket_unavailable = store.rpc_client.account_exists(
    &Pubkey::from_str(&ticket_nft)?,
    CommitmentConfig::processed()
  ).await?;

  if is_ticket_unavailable {
    return Err(Report::msg("Ticket unavailable"))?
  }

  if let SaleType::FixedPrice {amount} = sale.ticket_type.sale_type {
    calculate_price_and_fees(amount as i64, store.config.ticket_purchae_protocol_fee).await
  } else {
    return Err(Report::msg("Only fixed price ticket types are supported"))?
  }
}

pub fn pending_ticket_prefix() -> String {
  "pt_".to_owned()
}

pub fn pending_ticket_key(event_id: &str, ticket_nft: &str) -> String {
  format!("{:?}{:?}/{:?}", pending_ticket_prefix(), event_id, ticket_nft)
}
