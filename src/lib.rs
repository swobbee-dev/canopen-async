//! An async, `no_std` CANopen master library.
//!
//! Built around the roles a CANopen *client/master* needs when talking to
//! third-party nodes (battery BMSes, drives, sensors):
//!
//! - [`NodeClient`]: per-node frame dispatch — feed received frames in, get
//!   typed [`NodeEvent`]s (heartbeat, EMCY, PDO, SYNC) out
//! - [`SdoClient`]: expedited, segmented and block SDO transfers
//! - [`pdo`]: typed PDO reception via [`PdoPayload`] and [`PdoSlot`]
//! - [`EmcyMessage`]: emergency object parsing
//! - NMT master commands ([`NodeClient::send_nmt`])
//!
//! The library is transport-agnostic over the async [`embedded_can`] traits
//! and executor-agnostic (no embassy executor required; timeouts use
//! `embassy-time`).

#![cfg_attr(not(test), no_std)]
#![allow(async_fn_in_trait)]

mod emcy;
mod frame;
mod node;
pub mod pdo;
mod protocol;
mod sdo;

pub use emcy::EmcyMessage;
pub use frame::{NmtCommand, NmtState};
pub use node::{NodeClient, NodeEvent};
pub use pdo::{PdoPayload, PdoSlot};
pub use protocol::{BlockAck, BlockInit, BlockUploadInit, BlockUploadSegment, Segment};
pub use sdo::{SdoClient, SdoError, SeekFrom, StreamReader, StreamSeeker};
