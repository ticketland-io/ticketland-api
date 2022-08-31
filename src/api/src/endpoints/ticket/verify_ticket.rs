use std::str::FromStr;
use serde::{Deserialize};
use borsh::{
  BorshSerialize,
  BorshDeserialize,
};
use solana_sdk::{
  pubkey::Pubkey,
  signature::Signature,
  borsh::try_from_slice_unchecked,
};
use actix_web::{
  web::{Data, Json, Query},
  HttpResponse,
  http::header,
};
use crate::{
  utils::store::Store,
};

#[derive(Deserialize)]
pub struct Body {
  pub event_id: String,
  pub code_challenge: String,
  pub ticket_owner_pubkey: String,
  pub sig: String,
}

#[derive(Deserialize)]
pub struct QueryString {
  pub ticket_nft: String,
  pub return_url: String,
}

#[derive(BorshSerialize)]
struct VerifyTicketMsg<'a> {
  pub event_id: &'a str,
  pub code_challenge: &'a str,
  pub ticket_nft: &'a str,
}

async fn load_ticket_metadata_account<T>(store: &Data<Store>, account_key: &Pubkey) -> T
  where 
    T: borsh::de::BorshDeserialize
{
  let mut account_data = store.rpc_client.get_account_data(account_key).unwrap();
  // remove the Anchor account discriminator
  account_data.drain(0..8);
  try_from_slice_unchecked::<T>(&account_data).unwrap()
}

#[derive(BorshDeserialize)]
struct TicketMetadata {
  pub attended: bool,
  pub event_id: [u8; 32],
  pub seat_index: u32,
  pub sale: Pubkey,
  pub price_sold: u64,
  pub owner: Pubkey,
  pub metadata: Pubkey,
}

pub async fn exec(
  store: Data<Store>,
  body: Json<Body>,
  qs: Query<QueryString>,
) -> HttpResponse {
  // 1. recover the signer
  let raw_message = VerifyTicketMsg {
    event_id: &body.event_id,
    code_challenge: &body.code_challenge,
    ticket_nft: &qs.ticket_nft,
  };

  let mut message: Vec<u8> = Vec::new();
  raw_message.serialize(&mut message).unwrap();

  let sig = Signature::from_str(&body.sig).unwrap();
  let ticket_owner_pubkey = Pubkey::from_str(&body.ticket_owner_pubkey).unwrap();

  let return_url = if sig.verify(&ticket_owner_pubkey.to_bytes(), &message) {
    // 2. check that signer is the owner of the given ticket_nft 
    let ticket_metadata = load_ticket_metadata_account::<TicketMetadata>(
      &store, 
      &Pubkey::from_str(&qs.ticket_nft).unwrap()
    ).await;

    if ticket_metadata.owner == ticket_owner_pubkey {
      // TODO: sign a message and include sig in the return_url
      format!("{}/success", qs.return_url.clone())
    } else {
      format!("{}/error", qs.return_url.clone())
    }
  } else {
    format!("{}/error", qs.return_url.clone())
  };

  HttpResponse::MovedPermanently()
  .append_header((header::LOCATION, return_url))
  .finish()
}
