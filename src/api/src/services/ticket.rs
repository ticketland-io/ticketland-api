use std::str::FromStr;
use eyre::{Result};
use borsh::BorshSerialize;
use solana_sdk::{
  pubkey::Pubkey,
  signer::keypair::Keypair,
  signature::Signer,
  signature::Signature,
  keccak::hashv,
};
use actix_web::{
  web::{Data},
};
use program_artifacts::ticket_nft::account_data::TicketMetadata;
use crate::{
  utils::store::Store,
  error::Error,
};

#[derive(BorshSerialize)]
struct VerifyTicketMsg<'a> {
  pub event_id: &'a str,
  pub code_challenge: &'a str,
  pub ticket_metadata: &'a str,
}

#[derive(BorshSerialize)]
struct VerifyTicketResult<'a> {
  pub event_id: &'a str,
  pub code_challenge: &'a str,
  pub ticket_owner_pubkey: &'a str,
  pub ticket_metadata: &'a str,
}

fn sign_msg<'a>(msg: VerifyTicketResult<'a>) -> String {
  // TODO: this will be read from config.rs
  let signer = Keypair::from_base58_string(&"2jCJYqD2DYBK9wiQ55SLzV6umpcKzEpczQ3o6QyCXLLYahmoFD3GeuN2R36QR85BuqELSZTqHKAwMwC6ev9Nr75u");
  let mut message: Vec<u8> = Vec::new();
  msg.serialize(&mut message).unwrap();
  let message_hash = &hashv(&[&message]).0;

  bs58::encode(signer.sign_message(message_hash)).into_string()
}

pub async fn verify_ticket(
  store: &Data<Store>,
  event_id: &str,
  code_challenge: &str,
  ticket_metadata: &str,
  ticket_owner_pubkey: &str,
  sig: &str,
) -> Result<String> {
  // 1. recover the signer
  let raw_message = VerifyTicketMsg {
    event_id: &event_id,
    code_challenge: &code_challenge,
    ticket_metadata: &ticket_metadata,
  };

  let mut message: Vec<u8> = Vec::new();
  raw_message.serialize(&mut message).unwrap();
  let message_hash = &hashv(&[&message]).0;

  let sig = Signature::from_str(&sig).unwrap();
  let ticket_owner = Pubkey::from_str(&ticket_owner_pubkey).unwrap();

  if sig.verify(&ticket_owner.to_bytes(), message_hash) {
    // 2. check that signer is the owner of the given ticket_metadata 
    let ticket_metadata_account = store.rpc_client.get_anchor_account_data::<TicketMetadata>(
      &Pubkey::from_str(&ticket_metadata)?
    ).await?;

    if ticket_metadata_account.owner == ticket_owner {
      let sig = sign_msg(VerifyTicketResult {
        event_id: &event_id,
        code_challenge: &code_challenge,
        ticket_owner_pubkey: &ticket_owner_pubkey,
        ticket_metadata: &ticket_metadata,
      });

      Ok(sig)
    } else {
      return Err(Error::TicketVerificationError)?
    }
  } else {
    return Err(Error::TicketVerificationError)?
  }
}
