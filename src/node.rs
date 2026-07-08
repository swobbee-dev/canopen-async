//! Per-node frame dispatcher for the master role.
//!
//! [`NodeClient`] bundles the SDO client with COB-ID classification for one
//! remote node: an application RX task feeds every received frame to
//! [`NodeClient::handle_frame`] and matches on the returned [`NodeEvent`]
//! instead of hand-rolling COB-ID comparisons. SDO responses are consumed
//! internally; PDO payload decoding stays application-side (see
//! [`crate::pdo`]).

use embassy_time::Duration;
use embedded_can::{Frame, Id, asynch::CanTx};

use crate::frame::{self, CobId, NmtCommand, NmtState};
use crate::{EmcyMessage, SdoClient, SdoError};

/// A frame classified against one node's predefined connection set.
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum NodeEvent<'f> {
    /// Heartbeat or boot-up message (0x700 + node id)
    Heartbeat(NmtState),
    /// Emergency message from the node (0x080 + node id)
    Emcy(EmcyMessage),
    /// A PDO transmitted by the node (0x180/0x280/0x380/0x480 + node id);
    /// `num` is 1-based. Decode `data` via a [`crate::pdo::PdoPayload`].
    Tpdo { num: u8, data: &'f [u8] },
    /// SYNC broadcast (0x080)
    Sync,
    /// The frame does not belong to this node's predefined connection set
    /// (or is a message type this library does not handle); the caller may
    /// route or drop it.
    Unhandled,
}

/// SDO client plus frame dispatch for a single remote node.
pub struct NodeClient<FRAME, TX: CanTx<Frame = FRAME>> {
    /// The node's SDO client, for direct object access.
    pub sdo: SdoClient<FRAME, TX>,
    node_id: u8,
}

impl<FRAME: Frame, TX: CanTx<Frame = FRAME>> NodeClient<FRAME, TX> {
    /// Create a client for the node with the given id (1..=127).
    pub fn new(node_id: u8, tx: TX, sdo_timeout: Duration) -> Result<Self, SdoError<TX::Error>> {
        Ok(Self {
            sdo: SdoClient::new(node_id, tx, sdo_timeout)?,
            node_id,
        })
    }

    /// Process one received frame.
    ///
    /// SDO responses are consumed by the SDO client and yield `None`;
    /// everything else comes back classified as a [`NodeEvent`].
    /// Synchronous: safe to call from any RX context.
    pub fn handle_frame<'f>(&self, frame: &'f FRAME) -> Option<NodeEvent<'f>> {
        let Id::Standard(id) = frame.id() else {
            return Some(NodeEvent::Unhandled);
        };

        match frame::classify(id.as_raw(), self.node_id) {
            CobId::SdoTx => {
                self.sdo.process_frame(frame);
                None
            }
            CobId::Heartbeat => match frame.data().first() {
                Some(&state) => Some(NodeEvent::Heartbeat(NmtState::from(state))),
                // A heartbeat without a state byte is malformed
                None => Some(NodeEvent::Unhandled),
            },
            CobId::Emcy => Some(NodeEvent::Emcy(EmcyMessage::parse(frame.data()))),
            CobId::Tpdo { num } => Some(NodeEvent::Tpdo {
                num,
                data: frame.data(),
            }),
            CobId::Sync => Some(NodeEvent::Sync),
            CobId::Time | CobId::Nmt | CobId::Rpdo { .. } | CobId::SdoRx | CobId::Other => {
                Some(NodeEvent::Unhandled)
            }
        }
    }

    /// Send an NMT command addressed to this node.
    pub async fn send_nmt(&self, command: NmtCommand) -> Result<(), TX::Error> {
        self.sdo.send_nmt(command as u8).await
    }
}
