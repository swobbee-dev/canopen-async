//! Per-node frame dispatcher for the master role.
//!
//! [`NodeClient`] bundles the SDO client, heartbeat supervision and EMCY
//! monitoring for one remote node: an application RX task feeds every
//! received frame to [`NodeClient::handle_frame`] and matches on the
//! returned [`NodeEvent`] instead of hand-rolling COB-ID comparisons. SDO
//! responses are consumed internally; heartbeats and EMCYs additionally
//! update the built-in monitors. PDO payload decoding stays
//! application-side (see [`crate::pdo`]).

use embassy_sync::blocking_mutex::raw::RawMutex;
use embassy_time::Duration;
use embedded_can::{Frame, Id, StandardId, asynch::CanTx};

use crate::emcy::EmcyMonitor;
use crate::frame::{self, CobId, NmtCommand, NmtState};
use crate::heartbeat::HeartbeatMonitor;
use crate::pdo::PdoConfig;
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

/// SDO client, monitors and frame dispatch for a single remote node.
pub struct NodeClient<M: RawMutex, FRAME, TX: CanTx<Frame = FRAME>> {
    /// The node's SDO client, for direct object access.
    pub sdo: SdoClient<FRAME, TX>,
    /// Heartbeat supervision, fed automatically by [`Self::handle_frame`].
    pub heartbeat: HeartbeatMonitor<M>,
    /// Emergency monitoring, fed automatically by [`Self::handle_frame`].
    pub emcy: EmcyMonitor<M>,
    node_id: u8,
}

impl<M: RawMutex, FRAME: Frame, TX: CanTx<Frame = FRAME>> NodeClient<M, FRAME, TX> {
    /// Create a client for the node with the given id (1..=127).
    pub fn new(node_id: u8, tx: TX, sdo_timeout: Duration) -> Result<Self, SdoError<TX::Error>> {
        Ok(Self {
            sdo: SdoClient::new(node_id, tx, sdo_timeout)?,
            heartbeat: HeartbeatMonitor::new(),
            emcy: EmcyMonitor::new(),
            node_id,
        })
    }

    /// Process one received frame.
    ///
    /// SDO responses are consumed by the SDO client and yield `None`;
    /// everything else comes back classified as a [`NodeEvent`]. Heartbeats
    /// and EMCYs also update [`Self::heartbeat`] and [`Self::emcy`].
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
                Some(&state) => {
                    let state = NmtState::from(state);
                    self.heartbeat.on_heartbeat(state);
                    Some(NodeEvent::Heartbeat(state))
                }
                // A heartbeat without a state byte is malformed
                None => Some(NodeEvent::Unhandled),
            },
            CobId::Emcy => {
                let msg = EmcyMessage::parse(frame.data());
                self.emcy.on_emcy(msg);
                Some(NodeEvent::Emcy(msg))
            }
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

    /// Broadcast a SYNC object (COB-ID 0x080).
    ///
    /// Nodes with synchronous PDO transmission types respond by sampling and
    /// transmitting their TPDOs — e.g. to request process data more often
    /// than the producer's own period.
    pub async fn send_sync(&self) -> Result<(), TX::Error> {
        let frame = FRAME::new(Id::Standard(StandardId::new(0x080).unwrap()), &[]).unwrap();
        self.sdo.transmit_raw(&frame).await
    }

    /// Transmit an RPDO to this node (COB-ID 0x200/0x300/0x400/0x500 +
    /// node id; `num` is 1-based, `data` at most 8 bytes).
    pub async fn send_rpdo(&self, num: u8, data: &[u8]) -> Result<(), SdoError<TX::Error>> {
        if !(1..=4).contains(&num) || data.len() > 8 {
            return Err(SdoError::BufferSizeWrong);
        }
        let cob_id = 0x100 * (num as u16 + 1) + self.node_id as u16;
        let frame = FRAME::new(Id::Standard(StandardId::new(cob_id).unwrap()), data).unwrap();
        self.sdo.transmit_raw(&frame).await.map_err(SdoError::TxError)
    }

    /// Reconfigure one of the node's transmit PDOs (the PDOs it sends) via
    /// SDO, following the CiA 301 sequence: invalidate the COB-ID, write the
    /// communication parameters, rewrite the mapping, revalidate.
    pub async fn configure_tpdo(
        &self,
        num: u8,
        config: &PdoConfig<'_>,
    ) -> Result<(), SdoError<TX::Error>> {
        self.configure_pdo(0x1800, 0x1A00, 0x180, num, config).await
    }

    /// Reconfigure one of the node's receive PDOs (the PDOs it listens to,
    /// transmitted by us via [`Self::send_rpdo`]).
    pub async fn configure_rpdo(
        &self,
        num: u8,
        config: &PdoConfig<'_>,
    ) -> Result<(), SdoError<TX::Error>> {
        self.configure_pdo(0x1400, 0x1600, 0x200, num, config).await
    }

    async fn configure_pdo(
        &self,
        comm_base: u16,
        map_base: u16,
        cob_base: u16,
        num: u8,
        config: &PdoConfig<'_>,
    ) -> Result<(), SdoError<TX::Error>> {
        if !(1..=4).contains(&num) || config.mappings.len() > 8 {
            return Err(SdoError::BufferSizeWrong);
        }
        let offset = (num - 1) as u16;
        let comm = comm_base + offset;
        let map = map_base + offset;
        let cob_id = config
            .cob_id
            .unwrap_or((cob_base + offset * 0x100 + self.node_id as u16) as u32);

        // Invalidate the PDO while reconfiguring (COB-ID valid bit, bit 31)
        self.sdo
            .write_expedited(comm, 1, &(cob_id | 0x8000_0000).to_le_bytes())
            .await?;

        self.sdo
            .write_expedited(comm, 2, &[config.transmission_type])
            .await?;
        if let Some(inhibit) = config.inhibit_time {
            self.sdo.write_expedited(comm, 3, &inhibit.to_le_bytes()).await?;
        }
        if let Some(timer) = config.event_timer {
            self.sdo.write_expedited(comm, 5, &timer.to_le_bytes()).await?;
        }

        // Mapping may only change while the entry count is 0
        self.sdo.write_expedited(map, 0, &[0]).await?;
        for (i, entry) in config.mappings.iter().enumerate() {
            self.sdo
                .write_expedited(map, i as u8 + 1, &entry.raw().to_le_bytes())
                .await?;
        }
        self.sdo
            .write_expedited(map, 0, &[config.mappings.len() as u8])
            .await?;

        // Revalidate the PDO
        self.sdo.write_expedited(comm, 1, &cob_id.to_le_bytes()).await?;
        Ok(())
    }
}
