use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
  #[error("Ticket Verification Error")]
  TicketVerificationError,
}
