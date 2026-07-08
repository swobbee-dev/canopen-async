//! An async, `no_std` CANopen master library.
//!
//! Built around the roles a CANopen *client/master* needs when talking to
//! third-party nodes (battery BMSes, drives, sensors):
//!
//! - [`SdoClient`]: expedited, segmented and block SDO transfers
//! - [`EmcyMessage`]: emergency object parsing
//! - NMT master commands (via [`SdoClient`]'s `send_nmt_*` methods)
//!
//! The library is transport-agnostic over the async [`embedded_can`] traits
//! and executor-agnostic (no embassy executor required; timeouts use
//! `embassy-time`).

#![cfg_attr(not(test), no_std)]
#![allow(async_fn_in_trait)]

mod emcy;
mod frame;
mod protocol;
mod sdo;

pub use emcy::EmcyMessage;
pub use frame::{NmtCommand, NmtState};
pub use protocol::{BlockAck, BlockInit, BlockUploadInit, BlockUploadSegment, Segment};
pub use sdo::{SdoClient, SdoError, SeekFrom, StreamReader, StreamSeeker};
