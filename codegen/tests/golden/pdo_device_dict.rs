// Generated code! Do not edit!
// Object dictionary of `PdoDevice` (vendor `Test`)
// Source: `pdo_device.eds` (FileVersion 1, FileRevision 0)

use ::canopen_async::dict::*;

/// `RPDO communication parameter` (0x1400, RECORD)
pub mod rpdo_communication_parameter {
    use ::canopen_async::dict::*;
    /// `max sub-index` — ro, default `2`
    pub const MAX_SUB_INDEX: SdoEntry<u8> = SdoEntry::new(0x1400, 0x00, "max sub-index");
    /// `COB-ID used by RPDO` — rw, default `$NODEID+0x200`
    pub const COB_ID_USED_BY_RPDO: SdoEntry<u32> = SdoEntry::new(0x1400, 0x01, "COB-ID used by RPDO");
    /// `transmission type` — rw, default `255`
    pub const TRANSMISSION_TYPE: SdoEntry<u8> = SdoEntry::new(0x1400, 0x02, "transmission type");
}
/// `RPDO mapping parameter` (0x1600, RECORD)
pub mod rpdo_mapping_parameter {
    use ::canopen_async::dict::*;
    /// `Number of mapped objects` — rw, default `1`
    pub const NUMBER_OF_MAPPED_OBJECTS: SdoEntry<u8> = SdoEntry::new(0x1600, 0x00, "Number of mapped objects");
    /// `mapped object 1` — rw, default `0x20010010`
    pub const MAPPED_OBJECT_1: SdoEntry<u32> = SdoEntry::new(0x1600, 0x01, "mapped object 1");
}
/// `TPDO communication parameter` (0x1800, RECORD)
pub mod tpdo_communication_parameter {
    use ::canopen_async::dict::*;
    /// `max sub-index` — ro, default `2`
    pub const MAX_SUB_INDEX: SdoEntry<u8> = SdoEntry::new(0x1800, 0x00, "max sub-index");
    /// `COB-ID used by TPDO` — rw, default `$NODEID+0x180`
    pub const COB_ID_USED_BY_TPDO: SdoEntry<u32> = SdoEntry::new(0x1800, 0x01, "COB-ID used by TPDO");
    /// `transmission type` — rw, default `254`
    pub const TRANSMISSION_TYPE: SdoEntry<u8> = SdoEntry::new(0x1800, 0x02, "transmission type");
}
/// `TPDO communication parameter` (0x1801, RECORD)
pub mod tpdo_communication_parameter_2 {
    use ::canopen_async::dict::*;
    /// `max sub-index` — ro, default `1`
    pub const MAX_SUB_INDEX: SdoEntry<u8> = SdoEntry::new(0x1801, 0x00, "max sub-index");
    /// `COB-ID used by TPDO` — rw, default `$NODEID+0x280`
    pub const COB_ID_USED_BY_TPDO: SdoEntry<u32> = SdoEntry::new(0x1801, 0x01, "COB-ID used by TPDO");
}
/// `TPDO mapping parameter` (0x1A00, RECORD)
pub mod tpdo_mapping_parameter {
    use ::canopen_async::dict::*;
    /// `Number of mapped objects` — rw, default `2`
    pub const NUMBER_OF_MAPPED_OBJECTS: SdoEntry<u8> = SdoEntry::new(0x1A00, 0x00, "Number of mapped objects");
    /// `mapped object 1` — rw, default `0x20000110`
    pub const MAPPED_OBJECT_1: SdoEntry<u32> = SdoEntry::new(0x1A00, 0x01, "mapped object 1");
    /// `mapped object 2` — rw, default `0x20000220`
    pub const MAPPED_OBJECT_2: SdoEntry<u32> = SdoEntry::new(0x1A00, 0x02, "mapped object 2");
}
/// `TPDO mapping parameter` (0x1A01, RECORD)
pub mod tpdo_mapping_parameter_2 {
    use ::canopen_async::dict::*;
    /// `Number of mapped objects` — rw, default `1`
    pub const NUMBER_OF_MAPPED_OBJECTS: SdoEntry<u8> = SdoEntry::new(0x1A01, 0x00, "Number of mapped objects");
    /// `mapped object 1` — rw, default `0x20000308`
    pub const MAPPED_OBJECT_1: SdoEntry<u32> = SdoEntry::new(0x1A01, 0x01, "mapped object 1");
}
/// `Measurements` (0x2000, RECORD)
pub mod measurements {
    use ::canopen_async::dict::*;
    /// `Highest sub-index supported` — ro, default `3`
    pub const HIGHEST_SUB_INDEX_SUPPORTED: SdoEntry<u8> = SdoEntry::new(0x2000, 0x00, "Highest sub-index supported");
    /// `Voltage` — ro
    pub const VOLTAGE: SdoEntry<u16> = SdoEntry::new(0x2000, 0x01, "Voltage");
    /// `Current` — ro
    pub const CURRENT: SdoEntry<i32> = SdoEntry::new(0x2000, 0x02, "Current");
    /// `Flags` — ro
    pub const FLAGS: SdoEntry<u8> = SdoEntry::new(0x2000, 0x03, "Flags");
}
/// `Target current` — rww
pub const TARGET_CURRENT: SdoEntry<i16> = SdoEntry::new(0x2001, 0x00, "Target current");

/// Tpdo1 payload (transmitted by the node, PDO 1, 6 bytes)
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Tpdo1 {
    /// `Voltage` (0x2000:0x01)
    pub voltage: u16,
    /// `Current` (0x2000:0x02)
    pub current: i32,
}

impl ::canopen_async::pdo::PdoPayload for Tpdo1 {
    fn decode(data: &[u8]) -> Option<Self> {
        use ::canopen_async::dict::SdoScalar;
        if data.len() < 6 {
            return None;
        }
        Some(Self {
            voltage: <u16 as SdoScalar>::from_le_bytes(&data[0..2]),
            current: <i32 as SdoScalar>::from_le_bytes(&data[2..6]),
        })
    }
}

/// Tpdo2 payload (transmitted by the node, PDO 2, 1 bytes)
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Tpdo2 {
    /// `Flags` (0x2000:0x03)
    pub flags: u8,
}

impl ::canopen_async::pdo::PdoPayload for Tpdo2 {
    fn decode(data: &[u8]) -> Option<Self> {
        use ::canopen_async::dict::SdoScalar;
        if data.is_empty() {
            return None;
        }
        Some(Self {
            flags: <u8 as SdoScalar>::from_le_bytes(&data[0..1]),
        })
    }
}

/// Rpdo1 payload (received by the node, PDO 1, 2 bytes)
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rpdo1 {
    /// `Target current` (0x2001:0x00)
    pub target_current: i16,
}

impl Rpdo1 {
    /// Encode for transmission via `NodeClient::send_rpdo(1, ..)`.
    pub fn to_bytes(self) -> ([u8; 8], usize) {
        use ::canopen_async::dict::SdoScalar;
        let mut data = [0u8; 8];
        SdoScalar::write_le_bytes(self.target_current, &mut data[0..2]);
        (data, 2)
    }
}
