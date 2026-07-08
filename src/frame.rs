//! CANopen COB-ID structure and shared protocol types (CiA 301, 7.3.3).
//!
//! The 11-bit CAN identifier of the predefined connection set splits into a
//! 4-bit function code and a 7-bit node id: `cob_id = (fc << 7) | node_id`.

/// NMT command specifiers (CiA 301, 7.2.8.3.1).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u8)]
pub enum NmtCommand {
    Start = 1,
    Stop = 2,
    EnterPreOperational = 128,
    ResetNode = 129,
    ResetCommunication = 130,
}

/// NMT states as reported in heartbeat messages (CiA 301, 7.2.8.1).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum NmtState {
    Bootup,
    Stopped,
    Operational,
    PreOperational,
    /// A state byte outside the values defined by CiA 301.
    Unknown(u8),
}

impl From<u8> for NmtState {
    fn from(raw: u8) -> Self {
        // The heartbeat toggle bit (node guarding) lives in bit 7.
        match raw & 0x7F {
            0 => NmtState::Bootup,
            4 => NmtState::Stopped,
            5 => NmtState::Operational,
            127 => NmtState::PreOperational,
            other => NmtState::Unknown(other),
        }
    }
}

/// Classification of an 11-bit COB-ID against the predefined connection set
/// for one specific node.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub(crate) enum CobId {
    Sync,
    /// Emergency from the node (0x080 + node id)
    Emcy,
    /// Time stamp object
    Time,
    /// PDO transmitted BY the node (0x180/0x280/0x380/0x480 + node id);
    /// `num` is 1-based.
    Tpdo { num: u8 },
    /// PDO received by the node (0x200/0x300/0x400/0x500 + node id)
    Rpdo { num: u8 },
    /// SDO response from the node (0x580 + node id)
    SdoTx,
    /// SDO request towards the node (0x600 + node id)
    SdoRx,
    /// Heartbeat / boot-up / node guarding (0x700 + node id)
    Heartbeat,
    /// NMT command broadcast (COB-ID 0)
    Nmt,
    /// Anything that does not belong to this node's predefined set
    Other,
}

/// Classify a standard-frame COB-ID for the given node id.
pub(crate) fn classify(cob_id: u16, node_id: u8) -> CobId {
    match cob_id {
        0x000 => return CobId::Nmt,
        0x080 => return CobId::Sync,
        0x100 => return CobId::Time,
        _ => {}
    }

    if (cob_id & 0x7F) != node_id as u16 {
        return CobId::Other;
    }

    match cob_id >> 7 {
        0x1 => CobId::Emcy,
        0x3 => CobId::Tpdo { num: 1 },
        0x5 => CobId::Tpdo { num: 2 },
        0x7 => CobId::Tpdo { num: 3 },
        0x9 => CobId::Tpdo { num: 4 },
        0x4 => CobId::Rpdo { num: 1 },
        0x6 => CobId::Rpdo { num: 2 },
        0x8 => CobId::Rpdo { num: 3 },
        0xA => CobId::Rpdo { num: 4 },
        0xB => CobId::SdoTx,
        0xC => CobId::SdoRx,
        0xE => CobId::Heartbeat,
        _ => CobId::Other,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classify_lime_node_1() {
        // The COB-IDs of the lime battery (node id 1)
        assert_eq!(classify(0x181, 1), CobId::Tpdo { num: 1 });
        assert_eq!(classify(0x281, 1), CobId::Tpdo { num: 2 });
        assert_eq!(classify(0x381, 1), CobId::Tpdo { num: 3 });
        assert_eq!(classify(0x481, 1), CobId::Tpdo { num: 4 });
        assert_eq!(classify(0x701, 1), CobId::Heartbeat);
        assert_eq!(classify(0x081, 1), CobId::Emcy);
        assert_eq!(classify(0x581, 1), CobId::SdoTx);
        assert_eq!(classify(0x601, 1), CobId::SdoRx);
    }

    #[test]
    fn classify_enyring_node_0x7f() {
        // The enyring battery uses node id 0x7F; its heartbeat is 0x77F.
        assert_eq!(classify(0x77F, 0x7F), CobId::Heartbeat);
        assert_eq!(classify(0x5FF, 0x7F), CobId::SdoTx);
        assert_eq!(classify(0x0FF, 0x7F), CobId::Emcy);
    }

    #[test]
    fn classify_broadcasts_and_foreign_nodes() {
        assert_eq!(classify(0x000, 1), CobId::Nmt);
        assert_eq!(classify(0x080, 1), CobId::Sync);
        assert_eq!(classify(0x100, 1), CobId::Time);
        // Frames of another node are not classified into this node's set
        assert_eq!(classify(0x182, 1), CobId::Other);
        assert_eq!(classify(0x702, 1), CobId::Other);
    }

    #[test]
    fn nmt_state_from_heartbeat_byte() {
        assert_eq!(NmtState::from(0), NmtState::Bootup);
        assert_eq!(NmtState::from(4), NmtState::Stopped);
        assert_eq!(NmtState::from(5), NmtState::Operational);
        assert_eq!(NmtState::from(127), NmtState::PreOperational);
        // Toggle bit (node guarding) is masked out
        assert_eq!(NmtState::from(0x85), NmtState::Operational);
        assert_eq!(NmtState::from(3), NmtState::Unknown(3));
    }
}
