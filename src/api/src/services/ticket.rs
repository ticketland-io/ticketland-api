use std::str::FromStr;
use chrono::Duration;
use eyre::{Result};
use borsh::BorshSerialize;
use solana_sdk::{
  pubkey::Pubkey, signer::keypair::Keypair, signature::Signer, signature::Signature, keccak::hashv,
};
use actix_web::web::Data;
use ticketland_core::storage::redis::get_redis_ticket_attended_key;
use program_artifacts::ticket_nft::account_data::TicketMetadata;
use crate::{utils::store::Store, error::Error};

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

fn sign_msg<'a>(signer_key: &str, msg: VerifyTicketResult<'a>) -> String {
  let signer = Keypair::from_base58_string(signer_key);
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
  ticket_nft: &str,
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
    let ticket_metadata_account = store
      .rpc_client
      .get_anchor_account_data::<TicketMetadata>(&Pubkey::from_str(&ticket_metadata)?)
      .await?;

    if ticket_metadata_account.owner == ticket_owner {
      let sig = sign_msg(
        &store.config.ticket_verifier_priv_key,
        VerifyTicketResult {
          event_id: &event_id,
          code_challenge: &code_challenge,
          ticket_owner_pubkey: &ticket_owner_pubkey,
          ticket_metadata: &ticket_metadata,
        },
      );

      let lock = store
        .redlock
        .lock(
          ticket_nft.as_bytes(),
          Duration::seconds(5).num_milliseconds() as usize,
        )
        .await?;
      let mut redis = store.redis.lock().unwrap();
      let redis_key = get_redis_ticket_attended_key(event_id, ticket_metadata);

      // If key exists, it means someone has already attended this event
      if let Ok(_) = redis.get(&redis_key).await {
        return Err(Error::TicketVerificationError)?;
      }

      // Push to set_attended queue to send the tx
      store
        .set_attended_queue
        .on_set_attended(event_id.to_owned(), ticket_nft.to_owned())
        .await?;

      redis.set(&redis_key, &"1".to_owned()).await?;
      store.redlock.unlock(lock).await;

      Ok(sig)
    } else {
      return Err(Error::TicketVerificationError)?;
    }
  } else {
    return Err(Error::TicketVerificationError)?;
  }
}
