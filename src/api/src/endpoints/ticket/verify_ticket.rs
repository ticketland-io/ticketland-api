use std::str::FromStr;
use serde::{Serialize, Deserialize};
use borsh::{
  BorshSerialize,
  BorshDeserialize,
};
use api_helpers::services::http::bad_request_error;
use solana_sdk::{
  pubkey::Pubkey,
  signer::keypair::Keypair,
  signature::Signer,
  signature::Signature,
  borsh::try_from_slice_unchecked,
  keccak::hashv,
};
use actix_web::{
  web::{Data, Json, Path},
  HttpResponse,
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
pub struct Params {
  pub ticket_metadata: String,
}

#[derive(BorshSerialize)]
struct VerifyTicketMsg<'a> {
  pub event_id: &'a str,
  pub code_challenge: &'a str,
  pub ticket_metadata: &'a str,
}

async fn load_ticket_metadata_account<T>(store: &Data<Store>, account_key: &Pubkey) -> T
  where 
    T: borsh::de::BorshDeserialize
{
  let mut account_data = store.rpc_client.get_account_data(account_key).await.unwrap();
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

#[derive(BorshSerialize)]
struct VerifyTicketResult<'a> {
  pub event_id: &'a str,
  pub code_challenge: &'a str,
  pub ticket_metadata: &'a str,
}

fn sign_msg<'a>(msg: VerifyTicketResult<'a>) -> String {
  let signer = Keypair::from_base58_string(&"2jCJYqD2DYBK9wiQ55SLzV6umpcKzEpczQ3o6QyCXLLYahmoFD3GeuN2R36QR85BuqELSZTqHKAwMwC6ev9Nr75u");
  let mut message: Vec<u8> = Vec::new();
  msg.serialize(&mut message).unwrap();
  let message_hash = &hashv(&[&message]).0;

  hex::encode(signer.sign_message(message_hash))
}


#[derive(Serialize)]
pub struct Response {
  pub event_id: String,
  pub code_challenge: String,
  pub ticket_owner_pubkey: String,
  pub sig: String,
}

pub async fn exec(
  store: Data<Store>,
  body: Json<Body>,
  qs: Path<Params>,
) -> HttpResponse {
  // 1. recover the signer
  let raw_message = VerifyTicketMsg {
    event_id: &body.event_id,
    code_challenge: &body.code_challenge,
    ticket_metadata: &qs.ticket_metadata,
  };

  let mut message: Vec<u8> = Vec::new();
  raw_message.serialize(&mut message).unwrap();
  let message_hash = &hashv(&[&message]).0;

  let sig = Signature::from_str(&body.sig).unwrap();
  let ticket_owner_pubkey = Pubkey::from_str(&body.ticket_owner_pubkey).unwrap();

  let response = if sig.verify(&ticket_owner_pubkey.to_bytes(), message_hash) {
    // 2. check that signer is the owner of the given ticket_metadata 
    let ticket_metadata = load_ticket_metadata_account::<TicketMetadata>(
      &store, 
      &Pubkey::from_str(&qs.ticket_metadata).unwrap()
    ).await;

    if ticket_metadata.owner == ticket_owner_pubkey {
      let sig = sign_msg(VerifyTicketResult {
        event_id: &body.event_id,
        code_challenge: &body.code_challenge,
        ticket_metadata: &qs.ticket_metadata,
      });

      HttpResponse::Ok()
      .json(Response {
        event_id: body.event_id.clone(),
        code_challenge: body.code_challenge.clone(),
        ticket_owner_pubkey: body.ticket_owner_pubkey.clone(),
        sig,
      })
    } else {
      bad_request_error()
    }
  } else {
    bad_request_error()
  };

  response
}
