// Generated code! Do not edit!
// Object dictionary of `Mobius BCU` (vendor `Enyring`)
// Source: `bat_enyring.eds` (FileVersion 0.9.15, FileRevision 1)

use ::canopen_async::dict::*;

/// `Device type` — ro, default `0x00000000`
pub const DEVICE_TYPE: SdoEntry<u32> = SdoEntry::new(0x1000, 0x00, "Device type");
/// `Error register` — ro, default `0`
pub const ERROR_REGISTER: SdoEntry<u8> = SdoEntry::new(0x1001, 0x00, "Error register");
/// `Manufacturer status register` — ro, default `0`
pub const MANUFACTURER_STATUS_REGISTER: SdoEntry<u32> = SdoEntry::new(0x1002, 0x00, "Manufacturer status register");
/// `Pre-defined error field` (0x1003, ARRAY): 8 × u32
pub mod pre_defined_error_field {
    use ::canopen_async::dict::*;
    pub const ENTRIES: SdoArray<u32> = SdoArray::new(0x1003, 8, "Pre-defined error field");
    /// `Standard error field` — ro, default `0`
    pub const STANDARD_ERROR_FIELD: SdoEntry<u32> = SdoEntry::new(0x1003, 0x01, "Standard error field");
    /// `Standard error field` — ro, default `0`
    pub const STANDARD_ERROR_FIELD_2: SdoEntry<u32> = SdoEntry::new(0x1003, 0x02, "Standard error field");
    /// `Standard error field` — ro, default `0`
    pub const STANDARD_ERROR_FIELD_3: SdoEntry<u32> = SdoEntry::new(0x1003, 0x03, "Standard error field");
    /// `Standard error field` — ro, default `0`
    pub const STANDARD_ERROR_FIELD_4: SdoEntry<u32> = SdoEntry::new(0x1003, 0x04, "Standard error field");
    /// `Standard error field` — ro, default `0`
    pub const STANDARD_ERROR_FIELD_5: SdoEntry<u32> = SdoEntry::new(0x1003, 0x05, "Standard error field");
    /// `Standard error field` — ro, default `0`
    pub const STANDARD_ERROR_FIELD_6: SdoEntry<u32> = SdoEntry::new(0x1003, 0x06, "Standard error field");
    /// `Standard error field` — ro, default `0`
    pub const STANDARD_ERROR_FIELD_7: SdoEntry<u32> = SdoEntry::new(0x1003, 0x07, "Standard error field");
    /// `Standard error field` — ro, default `0`
    pub const STANDARD_ERROR_FIELD_8: SdoEntry<u32> = SdoEntry::new(0x1003, 0x08, "Standard error field");
}
/// `COB-ID SYNC message` — rw, default `0x00000080`
pub const COB_ID_SYNC_MESSAGE: SdoEntry<u32> = SdoEntry::new(0x1005, 0x00, "COB-ID SYNC message");
/// `Communication cycle period` — rw, default `0`
pub const COMMUNICATION_CYCLE_PERIOD: SdoEntry<u32> = SdoEntry::new(0x1006, 0x00, "Communication cycle period");
/// `Synchronous window length` — rw, default `0`
pub const SYNCHRONOUS_WINDOW_LENGTH: SdoEntry<u32> = SdoEntry::new(0x1007, 0x00, "Synchronous window length");
/// `Manufacturer device name` — const, default `CANopenNode`
pub const MANUFACTURER_DEVICE_NAME: SdoEntry<VisibleString> = SdoEntry::new(0x1008, 0x00, "Manufacturer device name");
/// `Manufacturer hardware version` — const, default `3.00`
pub const MANUFACTURER_HARDWARE_VERSION: SdoEntry<VisibleString> = SdoEntry::new(0x1009, 0x00, "Manufacturer hardware version");
/// `Manufacturer software version` — const, default `3.00`
pub const MANUFACTURER_SOFTWARE_VERSION: SdoEntry<VisibleString> = SdoEntry::new(0x100A, 0x00, "Manufacturer software version");
/// `Guard Time` — ro
pub const GUARD_TIME: SdoEntry<u16> = SdoEntry::new(0x100C, 0x00, "Guard Time");
/// `Life time factor` — ro
pub const LIFE_TIME_FACTOR: SdoEntry<u8> = SdoEntry::new(0x100D, 0x00, "Life time factor");
/// `Store parameters` (0x1010, ARRAY): 1 × u32
pub mod store_parameters {
    use ::canopen_async::dict::*;
    pub const ENTRIES: SdoArray<u32> = SdoArray::new(0x1010, 1, "Store parameters");
    /// `save all parameters` — rw, default `0x00000003`
    pub const SAVE_ALL_PARAMETERS: SdoEntry<u32> = SdoEntry::new(0x1010, 0x01, "save all parameters");
}
/// `Restore default parameters` (0x1011, ARRAY): 1 × u32
pub mod restore_default_parameters {
    use ::canopen_async::dict::*;
    pub const ENTRIES: SdoArray<u32> = SdoArray::new(0x1011, 1, "Restore default parameters");
    /// `restore all default parameters` — rw, default `0x00000001`
    pub const RESTORE_ALL_DEFAULT_PARAMETERS: SdoEntry<u32> = SdoEntry::new(0x1011, 0x01, "restore all default parameters");
}
/// `COB-ID TIME` — ro
pub const COB_ID_TIME: SdoEntry<u32> = SdoEntry::new(0x1012, 0x00, "COB-ID TIME");
/// `High resolution time stamp` — rw
pub const HIGH_RESOLUTION_TIME_STAMP: SdoEntry<u32> = SdoEntry::new(0x1013, 0x00, "High resolution time stamp");
/// `COB-ID EMCY` — ro, default `$NODEID+0x80`
pub const COB_ID_EMCY: SdoEntry<u32> = SdoEntry::new(0x1014, 0x00, "COB-ID EMCY");
/// `inhibit time EMCY` — rw, default `100`
pub const INHIBIT_TIME_EMCY: SdoEntry<u16> = SdoEntry::new(0x1015, 0x00, "inhibit time EMCY");
/// `Consumer heartbeat time` (0x1016, ARRAY): 4 × u32
pub mod consumer_heartbeat_time {
    use ::canopen_async::dict::*;
    pub const ENTRIES: SdoArray<u32> = SdoArray::new(0x1016, 4, "Consumer heartbeat time");
    /// `Consumer heartbeat time` — rw, default `0x00000000`
    pub const CONSUMER_HEARTBEAT_TIME: SdoEntry<u32> = SdoEntry::new(0x1016, 0x01, "Consumer heartbeat time");
    /// `Consumer heartbeat time` — rw, default `0x00000000`
    pub const CONSUMER_HEARTBEAT_TIME_2: SdoEntry<u32> = SdoEntry::new(0x1016, 0x02, "Consumer heartbeat time");
    /// `Consumer heartbeat time` — rw, default `0x00000000`
    pub const CONSUMER_HEARTBEAT_TIME_3: SdoEntry<u32> = SdoEntry::new(0x1016, 0x03, "Consumer heartbeat time");
    /// `Consumer heartbeat time` — rw, default `0x00000000`
    pub const CONSUMER_HEARTBEAT_TIME_4: SdoEntry<u32> = SdoEntry::new(0x1016, 0x04, "Consumer heartbeat time");
}
/// `Producer heartbeat time` — rw, default `300`
pub const PRODUCER_HEARTBEAT_TIME: SdoEntry<u16> = SdoEntry::new(0x1017, 0x00, "Producer heartbeat time");
/// `Identity` (0x1018, RECORD)
pub mod identity {
    use ::canopen_async::dict::*;
    /// `max sub-index` — ro, default `4`
    pub const MAX_SUB_INDEX: SdoEntry<u8> = SdoEntry::new(0x1018, 0x00, "max sub-index");
    /// `Vendor-ID` — ro, default `0x00000000`
    pub const VENDOR_ID: SdoEntry<u32> = SdoEntry::new(0x1018, 0x01, "Vendor-ID");
    /// `Product code` — ro, default `0x00000000`
    pub const PRODUCT_CODE: SdoEntry<u32> = SdoEntry::new(0x1018, 0x02, "Product code");
    /// `Revision number` — ro, default `0x00000000`
    pub const REVISION_NUMBER: SdoEntry<u32> = SdoEntry::new(0x1018, 0x03, "Revision number");
    /// `Serial number` — ro, default `0x00000000`
    pub const SERIAL_NUMBER: SdoEntry<u32> = SdoEntry::new(0x1018, 0x04, "Serial number");
}
/// `Synchronous counter overflow value` — rw, default `0`
pub const SYNCHRONOUS_COUNTER_OVERFLOW_VALUE: SdoEntry<u8> = SdoEntry::new(0x1019, 0x00, "Synchronous counter overflow value");
/// `Error behavior` (0x1029, ARRAY): 6 × u8
pub mod error_behavior {
    use ::canopen_async::dict::*;
    pub const ENTRIES: SdoArray<u8> = SdoArray::new(0x1029, 6, "Error behavior");
    /// `Communication` — rw, default `0x00`
    pub const COMMUNICATION: SdoEntry<u8> = SdoEntry::new(0x1029, 0x01, "Communication");
    /// `Communication other` — rw, default `0x00`
    pub const COMMUNICATION_OTHER: SdoEntry<u8> = SdoEntry::new(0x1029, 0x02, "Communication other");
    /// `Communication passive` — rw, default `0x01`
    pub const COMMUNICATION_PASSIVE: SdoEntry<u8> = SdoEntry::new(0x1029, 0x03, "Communication passive");
    /// `Generic` — rw, default `0x00`
    pub const GENERIC: SdoEntry<u8> = SdoEntry::new(0x1029, 0x04, "Generic");
    /// `Device profile` — rw, default `0x00`
    pub const DEVICE_PROFILE: SdoEntry<u8> = SdoEntry::new(0x1029, 0x05, "Device profile");
    /// `Manufacturer specific` — rw, default `0x00`
    pub const MANUFACTURER_SPECIFIC: SdoEntry<u8> = SdoEntry::new(0x1029, 0x06, "Manufacturer specific");
}
/// `SDO server parameter` (0x1200, RECORD)
pub mod sdo_server_parameter {
    use ::canopen_async::dict::*;
    /// `max sub-index` — ro, default `2`
    pub const MAX_SUB_INDEX: SdoEntry<u8> = SdoEntry::new(0x1200, 0x00, "max sub-index");
    /// `COB-ID client to server` — ro, default `$NODEID+0x600`
    pub const COB_ID_CLIENT_TO_SERVER: SdoEntry<u32> = SdoEntry::new(0x1200, 0x01, "COB-ID client to server");
    /// `COB-ID server to client` — ro, default `$NODEID+0x580`
    pub const COB_ID_SERVER_TO_CLIENT: SdoEntry<u32> = SdoEntry::new(0x1200, 0x02, "COB-ID server to client");
}
/// `SDO client parameter` (0x1280, RECORD)
pub mod sdo_client_parameter {
    use ::canopen_async::dict::*;
    /// `max sub-index` — ro, default `3`
    pub const MAX_SUB_INDEX: SdoEntry<u8> = SdoEntry::new(0x1280, 0x00, "max sub-index");
    /// `COB-ID client to server` — rw, default `0`
    pub const COB_ID_CLIENT_TO_SERVER: SdoEntry<u32> = SdoEntry::new(0x1280, 0x01, "COB-ID client to server");
    /// `COB-ID server to client` — rw, default `0`
    pub const COB_ID_SERVER_TO_CLIENT: SdoEntry<u32> = SdoEntry::new(0x1280, 0x02, "COB-ID server to client");
    /// `Node-ID of the SDO server` — rw, default `0`
    pub const NODE_ID_OF_THE_SDO_SERVER: SdoEntry<u8> = SdoEntry::new(0x1280, 0x03, "Node-ID of the SDO server");
}
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
    /// `Number of mapped objects` — rw, default `2`
    pub const NUMBER_OF_MAPPED_OBJECTS: SdoEntry<u8> = SdoEntry::new(0x1600, 0x00, "Number of mapped objects");
    /// `mapped object 1` — rw, default `0x00000000`
    pub const MAPPED_OBJECT_1: SdoEntry<u32> = SdoEntry::new(0x1600, 0x01, "mapped object 1");
    /// `mapped object 2` — rw, default `0x00000000`
    pub const MAPPED_OBJECT_2: SdoEntry<u32> = SdoEntry::new(0x1600, 0x02, "mapped object 2");
    /// `mapped object 3` — rw, default `0x00000000`
    pub const MAPPED_OBJECT_3: SdoEntry<u32> = SdoEntry::new(0x1600, 0x03, "mapped object 3");
    /// `mapped object 4` — rw, default `0x00000000`
    pub const MAPPED_OBJECT_4: SdoEntry<u32> = SdoEntry::new(0x1600, 0x04, "mapped object 4");
    /// `mapped object 5` — rw, default `0x00000000`
    pub const MAPPED_OBJECT_5: SdoEntry<u32> = SdoEntry::new(0x1600, 0x05, "mapped object 5");
    /// `mapped object 6` — rw, default `0x00000000`
    pub const MAPPED_OBJECT_6: SdoEntry<u32> = SdoEntry::new(0x1600, 0x06, "mapped object 6");
    /// `mapped object 7` — rw, default `0x00000000`
    pub const MAPPED_OBJECT_7: SdoEntry<u32> = SdoEntry::new(0x1600, 0x07, "mapped object 7");
    /// `mapped object 8` — rw, default `0x00000000`
    pub const MAPPED_OBJECT_8: SdoEntry<u32> = SdoEntry::new(0x1600, 0x08, "mapped object 8");
}
/// `TPDO communication parameter` (0x1800, RECORD)
pub mod tpdo_communication_parameter {
    use ::canopen_async::dict::*;
    /// `max sub-index` — ro, default `6`
    pub const MAX_SUB_INDEX: SdoEntry<u8> = SdoEntry::new(0x1800, 0x00, "max sub-index");
    /// `COB-ID used by TPDO` — rw, default `$NODEID+0x180`
    pub const COB_ID_USED_BY_TPDO: SdoEntry<u32> = SdoEntry::new(0x1800, 0x01, "COB-ID used by TPDO");
    /// `transmission type` — rw, default `255`
    pub const TRANSMISSION_TYPE: SdoEntry<u8> = SdoEntry::new(0x1800, 0x02, "transmission type");
    /// `inhibit time` — rw, default `100`
    pub const INHIBIT_TIME: SdoEntry<u16> = SdoEntry::new(0x1800, 0x03, "inhibit time");
    /// `compatibility entry` — ro, default `0`
    pub const COMPATIBILITY_ENTRY: SdoEntry<u8> = SdoEntry::new(0x1800, 0x04, "compatibility entry");
    /// `event timer` — rw, default `0`
    pub const EVENT_TIMER: SdoEntry<u16> = SdoEntry::new(0x1800, 0x05, "event timer");
    /// `SYNC start value` — rw, default `0`
    pub const SYNC_START_VALUE: SdoEntry<u8> = SdoEntry::new(0x1800, 0x06, "SYNC start value");
}
/// `TPDO mapping parameter` (0x1A00, RECORD)
pub mod tpdo_mapping_parameter {
    use ::canopen_async::dict::*;
    /// `Number of mapped objects` — rw, default `2`
    pub const NUMBER_OF_MAPPED_OBJECTS: SdoEntry<u8> = SdoEntry::new(0x1A00, 0x00, "Number of mapped objects");
    /// `mapped object 1` — rw, default `0x00000000`
    pub const MAPPED_OBJECT_1: SdoEntry<u32> = SdoEntry::new(0x1A00, 0x01, "mapped object 1");
    /// `mapped object 2` — rw, default `0x00000000`
    pub const MAPPED_OBJECT_2: SdoEntry<u32> = SdoEntry::new(0x1A00, 0x02, "mapped object 2");
    /// `mapped object 3` — rw, default `0x00000000`
    pub const MAPPED_OBJECT_3: SdoEntry<u32> = SdoEntry::new(0x1A00, 0x03, "mapped object 3");
    /// `mapped object 4` — rw, default `0x00000000`
    pub const MAPPED_OBJECT_4: SdoEntry<u32> = SdoEntry::new(0x1A00, 0x04, "mapped object 4");
    /// `mapped object 5` — rw, default `0x00000000`
    pub const MAPPED_OBJECT_5: SdoEntry<u32> = SdoEntry::new(0x1A00, 0x05, "mapped object 5");
    /// `mapped object 6` — rw, default `0x00000000`
    pub const MAPPED_OBJECT_6: SdoEntry<u32> = SdoEntry::new(0x1A00, 0x06, "mapped object 6");
    /// `mapped object 7` — rw, default `0x00000000`
    pub const MAPPED_OBJECT_7: SdoEntry<u32> = SdoEntry::new(0x1A00, 0x07, "mapped object 7");
    /// `mapped object 8` — rw, default `0x00000000`
    pub const MAPPED_OBJECT_8: SdoEntry<u32> = SdoEntry::new(0x1A00, 0x08, "mapped object 8");
}
/// `FW Update` — rw
pub const FW_UPDATE: SdoEntry<Domain> = SdoEntry::new(0x1F50, 0x00, "FW Update");
/// `NMTStartup` — rw
pub const NMT_STARTUP: SdoEntry<u32> = SdoEntry::new(0x1F80, 0x00, "NMTStartup");
/// `SlaveAssignment` (0x1F81, ARRAY): 127 × u32
pub mod slave_assignment {
    use ::canopen_async::dict::*;
    pub const ENTRIES: SdoArray<u32> = SdoArray::new(0x1F81, 127, "SlaveAssignment");
    /// `` — rw
    pub const SUB_1: SdoEntry<u32> = SdoEntry::new(0x1F81, 0x01, "");
    /// `` — rw
    pub const SUB_2: SdoEntry<u32> = SdoEntry::new(0x1F81, 0x02, "");
    /// `` — rw
    pub const SUB_3: SdoEntry<u32> = SdoEntry::new(0x1F81, 0x03, "");
    /// `` — rw
    pub const SUB_4: SdoEntry<u32> = SdoEntry::new(0x1F81, 0x04, "");
    /// `` — rw
    pub const SUB_5: SdoEntry<u32> = SdoEntry::new(0x1F81, 0x05, "");
    /// `` — rw
    pub const SUB_6: SdoEntry<u32> = SdoEntry::new(0x1F81, 0x06, "");
    /// `` — rw
    pub const SUB_7: SdoEntry<u32> = SdoEntry::new(0x1F81, 0x07, "");
    /// `` — rw
    pub const SUB_8: SdoEntry<u32> = SdoEntry::new(0x1F81, 0x08, "");
    /// `` — rw
    pub const SUB_9: SdoEntry<u32> = SdoEntry::new(0x1F81, 0x09, "");
    /// `` — rw
    pub const SUB_10: SdoEntry<u32> = SdoEntry::new(0x1F81, 0x0A, "");
    /// `` — rw
    pub const SUB_11: SdoEntry<u32> = SdoEntry::new(0x1F81, 0x0B, "");
    /// `` — rw
    pub const SUB_12: SdoEntry<u32> = SdoEntry::new(0x1F81, 0x0C, "");
    /// `` — rw
    pub const SUB_13: SdoEntry<u32> = SdoEntry::new(0x1F81, 0x0D, "");
    /// `` — rw
    pub const SUB_14: SdoEntry<u32> = SdoEntry::new(0x1F81, 0x0E, "");
    /// `` — rw
    pub const SUB_15: SdoEntry<u32> = SdoEntry::new(0x1F81, 0x0F, "");
    /// `` — rw
    pub const SUB_16: SdoEntry<u32> = SdoEntry::new(0x1F81, 0x10, "");
    /// `` — rw
    pub const SUB_17: SdoEntry<u32> = SdoEntry::new(0x1F81, 0x11, "");
    /// `` — rw
    pub const SUB_18: SdoEntry<u32> = SdoEntry::new(0x1F81, 0x12, "");
    /// `` — rw
    pub const SUB_19: SdoEntry<u32> = SdoEntry::new(0x1F81, 0x13, "");
    /// `` — rw
    pub const SUB_20: SdoEntry<u32> = SdoEntry::new(0x1F81, 0x14, "");
    /// `` — rw
    pub const SUB_21: SdoEntry<u32> = SdoEntry::new(0x1F81, 0x15, "");
    /// `` — rw
    pub const SUB_22: SdoEntry<u32> = SdoEntry::new(0x1F81, 0x16, "");
    /// `` — rw
    pub const SUB_23: SdoEntry<u32> = SdoEntry::new(0x1F81, 0x17, "");
    /// `` — rw
    pub const SUB_24: SdoEntry<u32> = SdoEntry::new(0x1F81, 0x18, "");
    /// `` — rw
    pub const SUB_25: SdoEntry<u32> = SdoEntry::new(0x1F81, 0x19, "");
    /// `` — rw
    pub const SUB_26: SdoEntry<u32> = SdoEntry::new(0x1F81, 0x1A, "");
    /// `` — rw
    pub const SUB_27: SdoEntry<u32> = SdoEntry::new(0x1F81, 0x1B, "");
    /// `` — rw
    pub const SUB_28: SdoEntry<u32> = SdoEntry::new(0x1F81, 0x1C, "");
    /// `` — rw
    pub const SUB_29: SdoEntry<u32> = SdoEntry::new(0x1F81, 0x1D, "");
    /// `` — rw
    pub const SUB_30: SdoEntry<u32> = SdoEntry::new(0x1F81, 0x1E, "");
    /// `` — rw
    pub const SUB_31: SdoEntry<u32> = SdoEntry::new(0x1F81, 0x1F, "");
    /// `` — rw
    pub const SUB_32: SdoEntry<u32> = SdoEntry::new(0x1F81, 0x20, "");
    /// `` — rw
    pub const SUB_33: SdoEntry<u32> = SdoEntry::new(0x1F81, 0x21, "");
    /// `` — rw
    pub const SUB_34: SdoEntry<u32> = SdoEntry::new(0x1F81, 0x22, "");
    /// `` — rw
    pub const SUB_35: SdoEntry<u32> = SdoEntry::new(0x1F81, 0x23, "");
    /// `` — rw
    pub const SUB_36: SdoEntry<u32> = SdoEntry::new(0x1F81, 0x24, "");
    /// `` — rw
    pub const SUB_37: SdoEntry<u32> = SdoEntry::new(0x1F81, 0x25, "");
    /// `` — rw
    pub const SUB_38: SdoEntry<u32> = SdoEntry::new(0x1F81, 0x26, "");
    /// `` — rw
    pub const SUB_39: SdoEntry<u32> = SdoEntry::new(0x1F81, 0x27, "");
    /// `` — rw
    pub const SUB_40: SdoEntry<u32> = SdoEntry::new(0x1F81, 0x28, "");
    /// `` — rw
    pub const SUB_41: SdoEntry<u32> = SdoEntry::new(0x1F81, 0x29, "");
    /// `` — rw
    pub const SUB_42: SdoEntry<u32> = SdoEntry::new(0x1F81, 0x2A, "");
    /// `` — rw
    pub const SUB_43: SdoEntry<u32> = SdoEntry::new(0x1F81, 0x2B, "");
    /// `` — rw
    pub const SUB_44: SdoEntry<u32> = SdoEntry::new(0x1F81, 0x2C, "");
    /// `` — rw
    pub const SUB_45: SdoEntry<u32> = SdoEntry::new(0x1F81, 0x2D, "");
    /// `` — rw
    pub const SUB_46: SdoEntry<u32> = SdoEntry::new(0x1F81, 0x2E, "");
    /// `` — rw
    pub const SUB_47: SdoEntry<u32> = SdoEntry::new(0x1F81, 0x2F, "");
    /// `` — rw
    pub const SUB_48: SdoEntry<u32> = SdoEntry::new(0x1F81, 0x30, "");
    /// `` — rw
    pub const SUB_49: SdoEntry<u32> = SdoEntry::new(0x1F81, 0x31, "");
    /// `` — rw
    pub const SUB_50: SdoEntry<u32> = SdoEntry::new(0x1F81, 0x32, "");
    /// `` — rw
    pub const SUB_51: SdoEntry<u32> = SdoEntry::new(0x1F81, 0x33, "");
    /// `` — rw
    pub const SUB_52: SdoEntry<u32> = SdoEntry::new(0x1F81, 0x34, "");
    /// `` — rw
    pub const SUB_53: SdoEntry<u32> = SdoEntry::new(0x1F81, 0x35, "");
    /// `` — rw
    pub const SUB_54: SdoEntry<u32> = SdoEntry::new(0x1F81, 0x36, "");
    /// `` — rw
    pub const SUB_55: SdoEntry<u32> = SdoEntry::new(0x1F81, 0x37, "");
    /// `` — rw
    pub const SUB_56: SdoEntry<u32> = SdoEntry::new(0x1F81, 0x38, "");
    /// `` — rw
    pub const SUB_57: SdoEntry<u32> = SdoEntry::new(0x1F81, 0x39, "");
    /// `` — rw
    pub const SUB_58: SdoEntry<u32> = SdoEntry::new(0x1F81, 0x3A, "");
    /// `` — rw
    pub const SUB_59: SdoEntry<u32> = SdoEntry::new(0x1F81, 0x3B, "");
    /// `` — rw
    pub const SUB_60: SdoEntry<u32> = SdoEntry::new(0x1F81, 0x3C, "");
    /// `` — rw
    pub const SUB_61: SdoEntry<u32> = SdoEntry::new(0x1F81, 0x3D, "");
    /// `` — rw
    pub const SUB_62: SdoEntry<u32> = SdoEntry::new(0x1F81, 0x3E, "");
    /// `` — rw
    pub const SUB_63: SdoEntry<u32> = SdoEntry::new(0x1F81, 0x3F, "");
    /// `` — rw
    pub const SUB_64: SdoEntry<u32> = SdoEntry::new(0x1F81, 0x40, "");
    /// `` — rw
    pub const SUB_65: SdoEntry<u32> = SdoEntry::new(0x1F81, 0x41, "");
    /// `` — rw
    pub const SUB_66: SdoEntry<u32> = SdoEntry::new(0x1F81, 0x42, "");
    /// `` — rw
    pub const SUB_67: SdoEntry<u32> = SdoEntry::new(0x1F81, 0x43, "");
    /// `` — rw
    pub const SUB_68: SdoEntry<u32> = SdoEntry::new(0x1F81, 0x44, "");
    /// `` — rw
    pub const SUB_69: SdoEntry<u32> = SdoEntry::new(0x1F81, 0x45, "");
    /// `` — rw
    pub const SUB_70: SdoEntry<u32> = SdoEntry::new(0x1F81, 0x46, "");
    /// `` — rw
    pub const SUB_71: SdoEntry<u32> = SdoEntry::new(0x1F81, 0x47, "");
    /// `` — rw
    pub const SUB_72: SdoEntry<u32> = SdoEntry::new(0x1F81, 0x48, "");
    /// `` — rw
    pub const SUB_73: SdoEntry<u32> = SdoEntry::new(0x1F81, 0x49, "");
    /// `` — rw
    pub const SUB_74: SdoEntry<u32> = SdoEntry::new(0x1F81, 0x4A, "");
    /// `` — rw
    pub const SUB_75: SdoEntry<u32> = SdoEntry::new(0x1F81, 0x4B, "");
    /// `` — rw
    pub const SUB_76: SdoEntry<u32> = SdoEntry::new(0x1F81, 0x4C, "");
    /// `` — rw
    pub const SUB_77: SdoEntry<u32> = SdoEntry::new(0x1F81, 0x4D, "");
    /// `` — rw
    pub const SUB_78: SdoEntry<u32> = SdoEntry::new(0x1F81, 0x4E, "");
    /// `` — rw
    pub const SUB_79: SdoEntry<u32> = SdoEntry::new(0x1F81, 0x4F, "");
    /// `` — rw
    pub const SUB_80: SdoEntry<u32> = SdoEntry::new(0x1F81, 0x50, "");
    /// `` — rw
    pub const SUB_81: SdoEntry<u32> = SdoEntry::new(0x1F81, 0x51, "");
    /// `` — rw
    pub const SUB_82: SdoEntry<u32> = SdoEntry::new(0x1F81, 0x52, "");
    /// `` — rw
    pub const SUB_83: SdoEntry<u32> = SdoEntry::new(0x1F81, 0x53, "");
    /// `` — rw
    pub const SUB_84: SdoEntry<u32> = SdoEntry::new(0x1F81, 0x54, "");
    /// `` — rw
    pub const SUB_85: SdoEntry<u32> = SdoEntry::new(0x1F81, 0x55, "");
    /// `` — rw
    pub const SUB_86: SdoEntry<u32> = SdoEntry::new(0x1F81, 0x56, "");
    /// `` — rw
    pub const SUB_87: SdoEntry<u32> = SdoEntry::new(0x1F81, 0x57, "");
    /// `` — rw
    pub const SUB_88: SdoEntry<u32> = SdoEntry::new(0x1F81, 0x58, "");
    /// `` — rw
    pub const SUB_89: SdoEntry<u32> = SdoEntry::new(0x1F81, 0x59, "");
    /// `` — rw
    pub const SUB_90: SdoEntry<u32> = SdoEntry::new(0x1F81, 0x5A, "");
    /// `` — rw
    pub const SUB_91: SdoEntry<u32> = SdoEntry::new(0x1F81, 0x5B, "");
    /// `` — rw
    pub const SUB_92: SdoEntry<u32> = SdoEntry::new(0x1F81, 0x5C, "");
    /// `` — rw
    pub const SUB_93: SdoEntry<u32> = SdoEntry::new(0x1F81, 0x5D, "");
    /// `` — rw
    pub const SUB_94: SdoEntry<u32> = SdoEntry::new(0x1F81, 0x5E, "");
    /// `` — rw
    pub const SUB_95: SdoEntry<u32> = SdoEntry::new(0x1F81, 0x5F, "");
    /// `` — rw
    pub const SUB_96: SdoEntry<u32> = SdoEntry::new(0x1F81, 0x60, "");
    /// `` — rw
    pub const SUB_97: SdoEntry<u32> = SdoEntry::new(0x1F81, 0x61, "");
    /// `` — rw
    pub const SUB_98: SdoEntry<u32> = SdoEntry::new(0x1F81, 0x62, "");
    /// `` — rw
    pub const SUB_99: SdoEntry<u32> = SdoEntry::new(0x1F81, 0x63, "");
    /// `` — rw
    pub const SUB_100: SdoEntry<u32> = SdoEntry::new(0x1F81, 0x64, "");
    /// `` — rw
    pub const SUB_101: SdoEntry<u32> = SdoEntry::new(0x1F81, 0x65, "");
    /// `` — rw
    pub const SUB_102: SdoEntry<u32> = SdoEntry::new(0x1F81, 0x66, "");
    /// `` — rw
    pub const SUB_103: SdoEntry<u32> = SdoEntry::new(0x1F81, 0x67, "");
    /// `` — rw
    pub const SUB_104: SdoEntry<u32> = SdoEntry::new(0x1F81, 0x68, "");
    /// `` — rw
    pub const SUB_105: SdoEntry<u32> = SdoEntry::new(0x1F81, 0x69, "");
    /// `` — rw
    pub const SUB_106: SdoEntry<u32> = SdoEntry::new(0x1F81, 0x6A, "");
    /// `` — rw
    pub const SUB_107: SdoEntry<u32> = SdoEntry::new(0x1F81, 0x6B, "");
    /// `` — rw
    pub const SUB_108: SdoEntry<u32> = SdoEntry::new(0x1F81, 0x6C, "");
    /// `` — rw
    pub const SUB_109: SdoEntry<u32> = SdoEntry::new(0x1F81, 0x6D, "");
    /// `` — rw
    pub const SUB_110: SdoEntry<u32> = SdoEntry::new(0x1F81, 0x6E, "");
    /// `` — rw
    pub const SUB_111: SdoEntry<u32> = SdoEntry::new(0x1F81, 0x6F, "");
    /// `` — rw
    pub const SUB_112: SdoEntry<u32> = SdoEntry::new(0x1F81, 0x70, "");
    /// `` — rw
    pub const SUB_113: SdoEntry<u32> = SdoEntry::new(0x1F81, 0x71, "");
    /// `` — rw
    pub const SUB_114: SdoEntry<u32> = SdoEntry::new(0x1F81, 0x72, "");
    /// `` — rw
    pub const SUB_115: SdoEntry<u32> = SdoEntry::new(0x1F81, 0x73, "");
    /// `` — rw
    pub const SUB_116: SdoEntry<u32> = SdoEntry::new(0x1F81, 0x74, "");
    /// `` — rw
    pub const SUB_117: SdoEntry<u32> = SdoEntry::new(0x1F81, 0x75, "");
    /// `` — rw
    pub const SUB_118: SdoEntry<u32> = SdoEntry::new(0x1F81, 0x76, "");
    /// `` — rw
    pub const SUB_119: SdoEntry<u32> = SdoEntry::new(0x1F81, 0x77, "");
    /// `` — rw
    pub const SUB_120: SdoEntry<u32> = SdoEntry::new(0x1F81, 0x78, "");
    /// `` — rw
    pub const SUB_121: SdoEntry<u32> = SdoEntry::new(0x1F81, 0x79, "");
    /// `` — rw
    pub const SUB_122: SdoEntry<u32> = SdoEntry::new(0x1F81, 0x7A, "");
    /// `` — rw
    pub const SUB_123: SdoEntry<u32> = SdoEntry::new(0x1F81, 0x7B, "");
    /// `` — rw
    pub const SUB_124: SdoEntry<u32> = SdoEntry::new(0x1F81, 0x7C, "");
    /// `` — rw
    pub const SUB_125: SdoEntry<u32> = SdoEntry::new(0x1F81, 0x7D, "");
    /// `` — rw
    pub const SUB_126: SdoEntry<u32> = SdoEntry::new(0x1F81, 0x7E, "");
    /// `` — rw
    pub const SUB_127: SdoEntry<u32> = SdoEntry::new(0x1F81, 0x7F, "");
}
/// `RequestNMT` (0x1F82, ARRAY): 127 × u8
pub mod request_nmt {
    use ::canopen_async::dict::*;
    pub const ENTRIES: SdoArray<u8> = SdoArray::new(0x1F82, 127, "RequestNMT");
    /// `` — rw
    pub const SUB_1: SdoEntry<u8> = SdoEntry::new(0x1F82, 0x01, "");
    /// `` — rw
    pub const SUB_2: SdoEntry<u8> = SdoEntry::new(0x1F82, 0x02, "");
    /// `` — rw
    pub const SUB_3: SdoEntry<u8> = SdoEntry::new(0x1F82, 0x03, "");
    /// `` — rw
    pub const SUB_4: SdoEntry<u8> = SdoEntry::new(0x1F82, 0x04, "");
    /// `` — rw
    pub const SUB_5: SdoEntry<u8> = SdoEntry::new(0x1F82, 0x05, "");
    /// `` — rw
    pub const SUB_6: SdoEntry<u8> = SdoEntry::new(0x1F82, 0x06, "");
    /// `` — rw
    pub const SUB_7: SdoEntry<u8> = SdoEntry::new(0x1F82, 0x07, "");
    /// `` — rw
    pub const SUB_8: SdoEntry<u8> = SdoEntry::new(0x1F82, 0x08, "");
    /// `` — rw
    pub const SUB_9: SdoEntry<u8> = SdoEntry::new(0x1F82, 0x09, "");
    /// `` — rw
    pub const SUB_10: SdoEntry<u8> = SdoEntry::new(0x1F82, 0x0A, "");
    /// `` — rw
    pub const SUB_11: SdoEntry<u8> = SdoEntry::new(0x1F82, 0x0B, "");
    /// `` — rw
    pub const SUB_12: SdoEntry<u8> = SdoEntry::new(0x1F82, 0x0C, "");
    /// `` — rw
    pub const SUB_13: SdoEntry<u8> = SdoEntry::new(0x1F82, 0x0D, "");
    /// `` — rw
    pub const SUB_14: SdoEntry<u8> = SdoEntry::new(0x1F82, 0x0E, "");
    /// `` — rw
    pub const SUB_15: SdoEntry<u8> = SdoEntry::new(0x1F82, 0x0F, "");
    /// `` — rw
    pub const SUB_16: SdoEntry<u8> = SdoEntry::new(0x1F82, 0x10, "");
    /// `` — rw
    pub const SUB_17: SdoEntry<u8> = SdoEntry::new(0x1F82, 0x11, "");
    /// `` — rw
    pub const SUB_18: SdoEntry<u8> = SdoEntry::new(0x1F82, 0x12, "");
    /// `` — rw
    pub const SUB_19: SdoEntry<u8> = SdoEntry::new(0x1F82, 0x13, "");
    /// `` — rw
    pub const SUB_20: SdoEntry<u8> = SdoEntry::new(0x1F82, 0x14, "");
    /// `` — rw
    pub const SUB_21: SdoEntry<u8> = SdoEntry::new(0x1F82, 0x15, "");
    /// `` — rw
    pub const SUB_22: SdoEntry<u8> = SdoEntry::new(0x1F82, 0x16, "");
    /// `` — rw
    pub const SUB_23: SdoEntry<u8> = SdoEntry::new(0x1F82, 0x17, "");
    /// `` — rw
    pub const SUB_24: SdoEntry<u8> = SdoEntry::new(0x1F82, 0x18, "");
    /// `` — rw
    pub const SUB_25: SdoEntry<u8> = SdoEntry::new(0x1F82, 0x19, "");
    /// `` — rw
    pub const SUB_26: SdoEntry<u8> = SdoEntry::new(0x1F82, 0x1A, "");
    /// `` — rw
    pub const SUB_27: SdoEntry<u8> = SdoEntry::new(0x1F82, 0x1B, "");
    /// `` — rw
    pub const SUB_28: SdoEntry<u8> = SdoEntry::new(0x1F82, 0x1C, "");
    /// `` — rw
    pub const SUB_29: SdoEntry<u8> = SdoEntry::new(0x1F82, 0x1D, "");
    /// `` — rw
    pub const SUB_30: SdoEntry<u8> = SdoEntry::new(0x1F82, 0x1E, "");
    /// `` — rw
    pub const SUB_31: SdoEntry<u8> = SdoEntry::new(0x1F82, 0x1F, "");
    /// `` — rw
    pub const SUB_32: SdoEntry<u8> = SdoEntry::new(0x1F82, 0x20, "");
    /// `` — rw
    pub const SUB_33: SdoEntry<u8> = SdoEntry::new(0x1F82, 0x21, "");
    /// `` — rw
    pub const SUB_34: SdoEntry<u8> = SdoEntry::new(0x1F82, 0x22, "");
    /// `` — rw
    pub const SUB_35: SdoEntry<u8> = SdoEntry::new(0x1F82, 0x23, "");
    /// `` — rw
    pub const SUB_36: SdoEntry<u8> = SdoEntry::new(0x1F82, 0x24, "");
    /// `` — rw
    pub const SUB_37: SdoEntry<u8> = SdoEntry::new(0x1F82, 0x25, "");
    /// `` — rw
    pub const SUB_38: SdoEntry<u8> = SdoEntry::new(0x1F82, 0x26, "");
    /// `` — rw
    pub const SUB_39: SdoEntry<u8> = SdoEntry::new(0x1F82, 0x27, "");
    /// `` — rw
    pub const SUB_40: SdoEntry<u8> = SdoEntry::new(0x1F82, 0x28, "");
    /// `` — rw
    pub const SUB_41: SdoEntry<u8> = SdoEntry::new(0x1F82, 0x29, "");
    /// `` — rw
    pub const SUB_42: SdoEntry<u8> = SdoEntry::new(0x1F82, 0x2A, "");
    /// `` — rw
    pub const SUB_43: SdoEntry<u8> = SdoEntry::new(0x1F82, 0x2B, "");
    /// `` — rw
    pub const SUB_44: SdoEntry<u8> = SdoEntry::new(0x1F82, 0x2C, "");
    /// `` — rw
    pub const SUB_45: SdoEntry<u8> = SdoEntry::new(0x1F82, 0x2D, "");
    /// `` — rw
    pub const SUB_46: SdoEntry<u8> = SdoEntry::new(0x1F82, 0x2E, "");
    /// `` — rw
    pub const SUB_47: SdoEntry<u8> = SdoEntry::new(0x1F82, 0x2F, "");
    /// `` — rw
    pub const SUB_48: SdoEntry<u8> = SdoEntry::new(0x1F82, 0x30, "");
    /// `` — rw
    pub const SUB_49: SdoEntry<u8> = SdoEntry::new(0x1F82, 0x31, "");
    /// `` — rw
    pub const SUB_50: SdoEntry<u8> = SdoEntry::new(0x1F82, 0x32, "");
    /// `` — rw
    pub const SUB_51: SdoEntry<u8> = SdoEntry::new(0x1F82, 0x33, "");
    /// `` — rw
    pub const SUB_52: SdoEntry<u8> = SdoEntry::new(0x1F82, 0x34, "");
    /// `` — rw
    pub const SUB_53: SdoEntry<u8> = SdoEntry::new(0x1F82, 0x35, "");
    /// `` — rw
    pub const SUB_54: SdoEntry<u8> = SdoEntry::new(0x1F82, 0x36, "");
    /// `` — rw
    pub const SUB_55: SdoEntry<u8> = SdoEntry::new(0x1F82, 0x37, "");
    /// `` — rw
    pub const SUB_56: SdoEntry<u8> = SdoEntry::new(0x1F82, 0x38, "");
    /// `` — rw
    pub const SUB_57: SdoEntry<u8> = SdoEntry::new(0x1F82, 0x39, "");
    /// `` — rw
    pub const SUB_58: SdoEntry<u8> = SdoEntry::new(0x1F82, 0x3A, "");
    /// `` — rw
    pub const SUB_59: SdoEntry<u8> = SdoEntry::new(0x1F82, 0x3B, "");
    /// `` — rw
    pub const SUB_60: SdoEntry<u8> = SdoEntry::new(0x1F82, 0x3C, "");
    /// `` — rw
    pub const SUB_61: SdoEntry<u8> = SdoEntry::new(0x1F82, 0x3D, "");
    /// `` — rw
    pub const SUB_62: SdoEntry<u8> = SdoEntry::new(0x1F82, 0x3E, "");
    /// `` — rw
    pub const SUB_63: SdoEntry<u8> = SdoEntry::new(0x1F82, 0x3F, "");
    /// `` — rw
    pub const SUB_64: SdoEntry<u8> = SdoEntry::new(0x1F82, 0x40, "");
    /// `` — rw
    pub const SUB_65: SdoEntry<u8> = SdoEntry::new(0x1F82, 0x41, "");
    /// `` — rw
    pub const SUB_66: SdoEntry<u8> = SdoEntry::new(0x1F82, 0x42, "");
    /// `` — rw
    pub const SUB_67: SdoEntry<u8> = SdoEntry::new(0x1F82, 0x43, "");
    /// `` — rw
    pub const SUB_68: SdoEntry<u8> = SdoEntry::new(0x1F82, 0x44, "");
    /// `` — rw
    pub const SUB_69: SdoEntry<u8> = SdoEntry::new(0x1F82, 0x45, "");
    /// `` — rw
    pub const SUB_70: SdoEntry<u8> = SdoEntry::new(0x1F82, 0x46, "");
    /// `` — rw
    pub const SUB_71: SdoEntry<u8> = SdoEntry::new(0x1F82, 0x47, "");
    /// `` — rw
    pub const SUB_72: SdoEntry<u8> = SdoEntry::new(0x1F82, 0x48, "");
    /// `` — rw
    pub const SUB_73: SdoEntry<u8> = SdoEntry::new(0x1F82, 0x49, "");
    /// `` — rw
    pub const SUB_74: SdoEntry<u8> = SdoEntry::new(0x1F82, 0x4A, "");
    /// `` — rw
    pub const SUB_75: SdoEntry<u8> = SdoEntry::new(0x1F82, 0x4B, "");
    /// `` — rw
    pub const SUB_76: SdoEntry<u8> = SdoEntry::new(0x1F82, 0x4C, "");
    /// `` — rw
    pub const SUB_77: SdoEntry<u8> = SdoEntry::new(0x1F82, 0x4D, "");
    /// `` — rw
    pub const SUB_78: SdoEntry<u8> = SdoEntry::new(0x1F82, 0x4E, "");
    /// `` — rw
    pub const SUB_79: SdoEntry<u8> = SdoEntry::new(0x1F82, 0x4F, "");
    /// `` — rw
    pub const SUB_80: SdoEntry<u8> = SdoEntry::new(0x1F82, 0x50, "");
    /// `` — rw
    pub const SUB_81: SdoEntry<u8> = SdoEntry::new(0x1F82, 0x51, "");
    /// `` — rw
    pub const SUB_82: SdoEntry<u8> = SdoEntry::new(0x1F82, 0x52, "");
    /// `` — rw
    pub const SUB_83: SdoEntry<u8> = SdoEntry::new(0x1F82, 0x53, "");
    /// `` — rw
    pub const SUB_84: SdoEntry<u8> = SdoEntry::new(0x1F82, 0x54, "");
    /// `` — rw
    pub const SUB_85: SdoEntry<u8> = SdoEntry::new(0x1F82, 0x55, "");
    /// `` — rw
    pub const SUB_86: SdoEntry<u8> = SdoEntry::new(0x1F82, 0x56, "");
    /// `` — rw
    pub const SUB_87: SdoEntry<u8> = SdoEntry::new(0x1F82, 0x57, "");
    /// `` — rw
    pub const SUB_88: SdoEntry<u8> = SdoEntry::new(0x1F82, 0x58, "");
    /// `` — rw
    pub const SUB_89: SdoEntry<u8> = SdoEntry::new(0x1F82, 0x59, "");
    /// `` — rw
    pub const SUB_90: SdoEntry<u8> = SdoEntry::new(0x1F82, 0x5A, "");
    /// `` — rw
    pub const SUB_91: SdoEntry<u8> = SdoEntry::new(0x1F82, 0x5B, "");
    /// `` — rw
    pub const SUB_92: SdoEntry<u8> = SdoEntry::new(0x1F82, 0x5C, "");
    /// `` — rw
    pub const SUB_93: SdoEntry<u8> = SdoEntry::new(0x1F82, 0x5D, "");
    /// `` — rw
    pub const SUB_94: SdoEntry<u8> = SdoEntry::new(0x1F82, 0x5E, "");
    /// `` — rw
    pub const SUB_95: SdoEntry<u8> = SdoEntry::new(0x1F82, 0x5F, "");
    /// `` — rw
    pub const SUB_96: SdoEntry<u8> = SdoEntry::new(0x1F82, 0x60, "");
    /// `` — rw
    pub const SUB_97: SdoEntry<u8> = SdoEntry::new(0x1F82, 0x61, "");
    /// `` — rw
    pub const SUB_98: SdoEntry<u8> = SdoEntry::new(0x1F82, 0x62, "");
    /// `` — rw
    pub const SUB_99: SdoEntry<u8> = SdoEntry::new(0x1F82, 0x63, "");
    /// `` — rw
    pub const SUB_100: SdoEntry<u8> = SdoEntry::new(0x1F82, 0x64, "");
    /// `` — rw
    pub const SUB_101: SdoEntry<u8> = SdoEntry::new(0x1F82, 0x65, "");
    /// `` — rw
    pub const SUB_102: SdoEntry<u8> = SdoEntry::new(0x1F82, 0x66, "");
    /// `` — rw
    pub const SUB_103: SdoEntry<u8> = SdoEntry::new(0x1F82, 0x67, "");
    /// `` — rw
    pub const SUB_104: SdoEntry<u8> = SdoEntry::new(0x1F82, 0x68, "");
    /// `` — rw
    pub const SUB_105: SdoEntry<u8> = SdoEntry::new(0x1F82, 0x69, "");
    /// `` — rw
    pub const SUB_106: SdoEntry<u8> = SdoEntry::new(0x1F82, 0x6A, "");
    /// `` — rw
    pub const SUB_107: SdoEntry<u8> = SdoEntry::new(0x1F82, 0x6B, "");
    /// `` — rw
    pub const SUB_108: SdoEntry<u8> = SdoEntry::new(0x1F82, 0x6C, "");
    /// `` — rw
    pub const SUB_109: SdoEntry<u8> = SdoEntry::new(0x1F82, 0x6D, "");
    /// `` — rw
    pub const SUB_110: SdoEntry<u8> = SdoEntry::new(0x1F82, 0x6E, "");
    /// `` — rw
    pub const SUB_111: SdoEntry<u8> = SdoEntry::new(0x1F82, 0x6F, "");
    /// `` — rw
    pub const SUB_112: SdoEntry<u8> = SdoEntry::new(0x1F82, 0x70, "");
    /// `` — rw
    pub const SUB_113: SdoEntry<u8> = SdoEntry::new(0x1F82, 0x71, "");
    /// `` — rw
    pub const SUB_114: SdoEntry<u8> = SdoEntry::new(0x1F82, 0x72, "");
    /// `` — rw
    pub const SUB_115: SdoEntry<u8> = SdoEntry::new(0x1F82, 0x73, "");
    /// `` — rw
    pub const SUB_116: SdoEntry<u8> = SdoEntry::new(0x1F82, 0x74, "");
    /// `` — rw
    pub const SUB_117: SdoEntry<u8> = SdoEntry::new(0x1F82, 0x75, "");
    /// `` — rw
    pub const SUB_118: SdoEntry<u8> = SdoEntry::new(0x1F82, 0x76, "");
    /// `` — rw
    pub const SUB_119: SdoEntry<u8> = SdoEntry::new(0x1F82, 0x77, "");
    /// `` — rw
    pub const SUB_120: SdoEntry<u8> = SdoEntry::new(0x1F82, 0x78, "");
    /// `` — rw
    pub const SUB_121: SdoEntry<u8> = SdoEntry::new(0x1F82, 0x79, "");
    /// `` — rw
    pub const SUB_122: SdoEntry<u8> = SdoEntry::new(0x1F82, 0x7A, "");
    /// `` — rw
    pub const SUB_123: SdoEntry<u8> = SdoEntry::new(0x1F82, 0x7B, "");
    /// `` — rw
    pub const SUB_124: SdoEntry<u8> = SdoEntry::new(0x1F82, 0x7C, "");
    /// `` — rw
    pub const SUB_125: SdoEntry<u8> = SdoEntry::new(0x1F82, 0x7D, "");
    /// `` — rw
    pub const SUB_126: SdoEntry<u8> = SdoEntry::new(0x1F82, 0x7E, "");
    /// `` — rw
    pub const SUB_127: SdoEntry<u8> = SdoEntry::new(0x1F82, 0x7F, "");
}
/// `BootTime` — rw
pub const BOOT_TIME: SdoEntry<u32> = SdoEntry::new(0x1F89, 0x00, "BootTime");
/// `Error status bits` — ro, default `00000000000000000000`
pub const ERROR_STATUS_BITS: SdoEntry<OctetString> = SdoEntry::new(0x2100, 0x00, "Error status bits");
/// `Status Word` — ro, default `0`
pub const STATUS_WORD: SdoEntry<u32> = SdoEntry::new(0x4201, 0x00, "Status Word");
/// `Control Word` — rww, default `0`
pub const CONTROL_WORD: SdoEntry<u32> = SdoEntry::new(0x4202, 0x00, "Control Word");
/// `Node ID` — rw, default `127`
pub const NODE_ID: SdoEntry<u8> = SdoEntry::new(0x4203, 0x00, "Node ID");
/// `Random number` (0x4204, ARRAY): 3 × u32
pub mod random_number {
    use ::canopen_async::dict::*;
    pub const ENTRIES: SdoArray<u32> = SdoArray::new(0x4204, 3, "Random number");
    /// `Bike` — ro, default `0`
    pub const BIKE: SdoEntry<u32> = SdoEntry::new(0x4204, 0x01, "Bike");
    /// `Station` — ro, default `0`
    pub const STATION: SdoEntry<u32> = SdoEntry::new(0x4204, 0x02, "Station");
    /// `Charger` — ro, default `0`
    pub const CHARGER: SdoEntry<u32> = SdoEntry::new(0x4204, 0x03, "Charger");
}
/// `Identification` (0x4205, ARRAY): 3 × u32
pub mod identification {
    use ::canopen_async::dict::*;
    pub const ENTRIES: SdoArray<u32> = SdoArray::new(0x4205, 3, "Identification");
    /// `Bike` — rw, default `0`
    pub const BIKE: SdoEntry<u32> = SdoEntry::new(0x4205, 0x01, "Bike");
    /// `Station` — rw, default `0`
    pub const STATION: SdoEntry<u32> = SdoEntry::new(0x4205, 0x02, "Station");
    /// `Charger` — rw, default `0`
    pub const CHARGER: SdoEntry<u32> = SdoEntry::new(0x4205, 0x03, "Charger");
}
/// `Bike UID attempt` (0x4206, ARRAY): 3 × u32
pub mod bike_uid_attempt {
    use ::canopen_async::dict::*;
    pub const ENTRIES: SdoArray<u32> = SdoArray::new(0x4206, 3, "Bike UID attempt");
    /// `UID1` — rw, default `0`
    pub const UID1: SdoEntry<u32> = SdoEntry::new(0x4206, 0x01, "UID1");
    /// `UID2` — rw, default `0`
    pub const UID2: SdoEntry<u32> = SdoEntry::new(0x4206, 0x02, "UID2");
    /// `UID3` — rw, default `0`
    pub const UID3: SdoEntry<u32> = SdoEntry::new(0x4206, 0x03, "UID3");
}
/// `Charger UID attempt` (0x4207, ARRAY): 3 × u32
pub mod charger_uid_attempt {
    use ::canopen_async::dict::*;
    pub const ENTRIES: SdoArray<u32> = SdoArray::new(0x4207, 3, "Charger UID attempt");
    /// `UID1` — rw, default `0`
    pub const UID1: SdoEntry<u32> = SdoEntry::new(0x4207, 0x01, "UID1");
    /// `UID2` — rw, default `0`
    pub const UID2: SdoEntry<u32> = SdoEntry::new(0x4207, 0x02, "UID2");
    /// `UID3` — rw, default `0`
    pub const UID3: SdoEntry<u32> = SdoEntry::new(0x4207, 0x03, "UID3");
}
/// `Connected Bike UID` — wo, default `000000000000`
pub const CONNECTED_BIKE_UID: SdoEntry<VisibleString> = SdoEntry::new(0x4208, 0x00, "Connected Bike UID");
/// `Connected Charger UID` — wo, default `000000000000`
pub const CONNECTED_CHARGER_UID: SdoEntry<VisibleString> = SdoEntry::new(0x4209, 0x00, "Connected Charger UID");
/// `IOT presence` (0x4210, ARRAY): 2 × i8
pub mod iot_presence {
    use ::canopen_async::dict::*;
    pub const ENTRIES: SdoArray<i8> = SdoArray::new(0x4210, 2, "IOT presence");
    /// `bike 1` — rw, default `0`
    pub const BIKE_1: SdoEntry<i8> = SdoEntry::new(0x4210, 0x01, "bike 1");
    /// `bike 2` — rw, default `0`
    pub const BIKE_2: SdoEntry<i8> = SdoEntry::new(0x4210, 0x02, "bike 2");
}
/// `Accepted bikes` (0x4211, ARRAY): 100 × Domain
pub mod accepted_bikes {
    use ::canopen_async::dict::*;
    pub const ENTRIES: SdoArray<Domain> = SdoArray::new(0x4211, 100, "Accepted bikes");
    /// `Bike 1` — rw
    pub const BIKE_1: SdoEntry<Domain> = SdoEntry::new(0x4211, 0x01, "Bike 1");
    /// `Bike 2` — rw
    pub const BIKE_2: SdoEntry<Domain> = SdoEntry::new(0x4211, 0x02, "Bike 2");
    /// `Bike 3` — rw
    pub const BIKE_3: SdoEntry<Domain> = SdoEntry::new(0x4211, 0x03, "Bike 3");
    /// `Bike 4` — rw
    pub const BIKE_4: SdoEntry<Domain> = SdoEntry::new(0x4211, 0x04, "Bike 4");
    /// `Bike 5` — rw
    pub const BIKE_5: SdoEntry<Domain> = SdoEntry::new(0x4211, 0x05, "Bike 5");
    /// `Bike 6` — rw
    pub const BIKE_6: SdoEntry<Domain> = SdoEntry::new(0x4211, 0x06, "Bike 6");
    /// `Bike 7` — rw
    pub const BIKE_7: SdoEntry<Domain> = SdoEntry::new(0x4211, 0x07, "Bike 7");
    /// `Bike 8` — rw
    pub const BIKE_8: SdoEntry<Domain> = SdoEntry::new(0x4211, 0x08, "Bike 8");
    /// `Bike 9` — rw
    pub const BIKE_9: SdoEntry<Domain> = SdoEntry::new(0x4211, 0x09, "Bike 9");
    /// `Bike 10` — rw
    pub const BIKE_10: SdoEntry<Domain> = SdoEntry::new(0x4211, 0x0A, "Bike 10");
    /// `Bike 11` — rw
    pub const BIKE_11: SdoEntry<Domain> = SdoEntry::new(0x4211, 0x0B, "Bike 11");
    /// `Bike 12` — rw
    pub const BIKE_12: SdoEntry<Domain> = SdoEntry::new(0x4211, 0x0C, "Bike 12");
    /// `Bike 13` — rw
    pub const BIKE_13: SdoEntry<Domain> = SdoEntry::new(0x4211, 0x0D, "Bike 13");
    /// `Bike 14` — rw
    pub const BIKE_14: SdoEntry<Domain> = SdoEntry::new(0x4211, 0x0E, "Bike 14");
    /// `Bike 15` — rw
    pub const BIKE_15: SdoEntry<Domain> = SdoEntry::new(0x4211, 0x0F, "Bike 15");
    /// `Bike 16` — rw
    pub const BIKE_16: SdoEntry<Domain> = SdoEntry::new(0x4211, 0x10, "Bike 16");
    /// `Bike 17` — rw
    pub const BIKE_17: SdoEntry<Domain> = SdoEntry::new(0x4211, 0x11, "Bike 17");
    /// `Bike 18` — rw
    pub const BIKE_18: SdoEntry<Domain> = SdoEntry::new(0x4211, 0x12, "Bike 18");
    /// `Bike 19` — rw
    pub const BIKE_19: SdoEntry<Domain> = SdoEntry::new(0x4211, 0x13, "Bike 19");
    /// `Bike 20` — rw
    pub const BIKE_20: SdoEntry<Domain> = SdoEntry::new(0x4211, 0x14, "Bike 20");
    /// `Bike 21` — rw
    pub const BIKE_21: SdoEntry<Domain> = SdoEntry::new(0x4211, 0x15, "Bike 21");
    /// `Bike 22` — rw
    pub const BIKE_22: SdoEntry<Domain> = SdoEntry::new(0x4211, 0x16, "Bike 22");
    /// `Bike 23` — rw
    pub const BIKE_23: SdoEntry<Domain> = SdoEntry::new(0x4211, 0x17, "Bike 23");
    /// `Bike 24` — rw
    pub const BIKE_24: SdoEntry<Domain> = SdoEntry::new(0x4211, 0x18, "Bike 24");
    /// `Bike 25` — rw
    pub const BIKE_25: SdoEntry<Domain> = SdoEntry::new(0x4211, 0x19, "Bike 25");
    /// `Bike 26` — rw
    pub const BIKE_26: SdoEntry<Domain> = SdoEntry::new(0x4211, 0x1A, "Bike 26");
    /// `Bike 27` — rw
    pub const BIKE_27: SdoEntry<Domain> = SdoEntry::new(0x4211, 0x1B, "Bike 27");
    /// `Bike 28` — rw
    pub const BIKE_28: SdoEntry<Domain> = SdoEntry::new(0x4211, 0x1C, "Bike 28");
    /// `Bike 29` — rw
    pub const BIKE_29: SdoEntry<Domain> = SdoEntry::new(0x4211, 0x1D, "Bike 29");
    /// `Bike 30` — rw
    pub const BIKE_30: SdoEntry<Domain> = SdoEntry::new(0x4211, 0x1E, "Bike 30");
    /// `Bike 31` — rw
    pub const BIKE_31: SdoEntry<Domain> = SdoEntry::new(0x4211, 0x1F, "Bike 31");
    /// `Bike 32` — rw
    pub const BIKE_32: SdoEntry<Domain> = SdoEntry::new(0x4211, 0x20, "Bike 32");
    /// `Bike 33` — rw
    pub const BIKE_33: SdoEntry<Domain> = SdoEntry::new(0x4211, 0x21, "Bike 33");
    /// `Bike 34` — rw
    pub const BIKE_34: SdoEntry<Domain> = SdoEntry::new(0x4211, 0x22, "Bike 34");
    /// `Bike 35` — rw
    pub const BIKE_35: SdoEntry<Domain> = SdoEntry::new(0x4211, 0x23, "Bike 35");
    /// `Bike 36` — rw
    pub const BIKE_36: SdoEntry<Domain> = SdoEntry::new(0x4211, 0x24, "Bike 36");
    /// `Bike 37` — rw
    pub const BIKE_37: SdoEntry<Domain> = SdoEntry::new(0x4211, 0x25, "Bike 37");
    /// `Bike 38` — rw
    pub const BIKE_38: SdoEntry<Domain> = SdoEntry::new(0x4211, 0x26, "Bike 38");
    /// `Bike 39` — rw
    pub const BIKE_39: SdoEntry<Domain> = SdoEntry::new(0x4211, 0x27, "Bike 39");
    /// `Bike 40` — rw
    pub const BIKE_40: SdoEntry<Domain> = SdoEntry::new(0x4211, 0x28, "Bike 40");
    /// `Bike 41` — rw
    pub const BIKE_41: SdoEntry<Domain> = SdoEntry::new(0x4211, 0x29, "Bike 41");
    /// `Bike 42` — rw
    pub const BIKE_42: SdoEntry<Domain> = SdoEntry::new(0x4211, 0x2A, "Bike 42");
    /// `Bike 43` — rw
    pub const BIKE_43: SdoEntry<Domain> = SdoEntry::new(0x4211, 0x2B, "Bike 43");
    /// `Bike 44` — rw
    pub const BIKE_44: SdoEntry<Domain> = SdoEntry::new(0x4211, 0x2C, "Bike 44");
    /// `Bike 45` — rw
    pub const BIKE_45: SdoEntry<Domain> = SdoEntry::new(0x4211, 0x2D, "Bike 45");
    /// `Bike 46` — rw
    pub const BIKE_46: SdoEntry<Domain> = SdoEntry::new(0x4211, 0x2E, "Bike 46");
    /// `Bike 47` — rw
    pub const BIKE_47: SdoEntry<Domain> = SdoEntry::new(0x4211, 0x2F, "Bike 47");
    /// `Bike 48` — rw
    pub const BIKE_48: SdoEntry<Domain> = SdoEntry::new(0x4211, 0x30, "Bike 48");
    /// `Bike 49` — rw
    pub const BIKE_49: SdoEntry<Domain> = SdoEntry::new(0x4211, 0x31, "Bike 49");
    /// `Bike 50` — rw
    pub const BIKE_50: SdoEntry<Domain> = SdoEntry::new(0x4211, 0x32, "Bike 50");
    /// `Bike 51` — rw
    pub const BIKE_51: SdoEntry<Domain> = SdoEntry::new(0x4211, 0x33, "Bike 51");
    /// `Bike 52` — rw
    pub const BIKE_52: SdoEntry<Domain> = SdoEntry::new(0x4211, 0x34, "Bike 52");
    /// `Bike 53` — rw
    pub const BIKE_53: SdoEntry<Domain> = SdoEntry::new(0x4211, 0x35, "Bike 53");
    /// `Bike 54` — rw
    pub const BIKE_54: SdoEntry<Domain> = SdoEntry::new(0x4211, 0x36, "Bike 54");
    /// `Bike 55` — rw
    pub const BIKE_55: SdoEntry<Domain> = SdoEntry::new(0x4211, 0x37, "Bike 55");
    /// `Bike 56` — rw
    pub const BIKE_56: SdoEntry<Domain> = SdoEntry::new(0x4211, 0x38, "Bike 56");
    /// `Bike 57` — rw
    pub const BIKE_57: SdoEntry<Domain> = SdoEntry::new(0x4211, 0x39, "Bike 57");
    /// `Bike 58` — rw
    pub const BIKE_58: SdoEntry<Domain> = SdoEntry::new(0x4211, 0x3A, "Bike 58");
    /// `Bike 59` — rw
    pub const BIKE_59: SdoEntry<Domain> = SdoEntry::new(0x4211, 0x3B, "Bike 59");
    /// `Bike 60` — rw
    pub const BIKE_60: SdoEntry<Domain> = SdoEntry::new(0x4211, 0x3C, "Bike 60");
    /// `Bike 61` — rw
    pub const BIKE_61: SdoEntry<Domain> = SdoEntry::new(0x4211, 0x3D, "Bike 61");
    /// `Bike 62` — rw
    pub const BIKE_62: SdoEntry<Domain> = SdoEntry::new(0x4211, 0x3E, "Bike 62");
    /// `Bike 63` — rw
    pub const BIKE_63: SdoEntry<Domain> = SdoEntry::new(0x4211, 0x3F, "Bike 63");
    /// `Bike 64` — rw
    pub const BIKE_64: SdoEntry<Domain> = SdoEntry::new(0x4211, 0x40, "Bike 64");
    /// `Bike 65` — rw
    pub const BIKE_65: SdoEntry<Domain> = SdoEntry::new(0x4211, 0x41, "Bike 65");
    /// `Bike 66` — rw
    pub const BIKE_66: SdoEntry<Domain> = SdoEntry::new(0x4211, 0x42, "Bike 66");
    /// `Bike 67` — rw
    pub const BIKE_67: SdoEntry<Domain> = SdoEntry::new(0x4211, 0x43, "Bike 67");
    /// `Bike 68` — rw
    pub const BIKE_68: SdoEntry<Domain> = SdoEntry::new(0x4211, 0x44, "Bike 68");
    /// `Bike 69` — rw
    pub const BIKE_69: SdoEntry<Domain> = SdoEntry::new(0x4211, 0x45, "Bike 69");
    /// `Bike 70` — rw
    pub const BIKE_70: SdoEntry<Domain> = SdoEntry::new(0x4211, 0x46, "Bike 70");
    /// `Bike 71` — rw
    pub const BIKE_71: SdoEntry<Domain> = SdoEntry::new(0x4211, 0x47, "Bike 71");
    /// `Bike 72` — rw
    pub const BIKE_72: SdoEntry<Domain> = SdoEntry::new(0x4211, 0x48, "Bike 72");
    /// `Bike 73` — rw
    pub const BIKE_73: SdoEntry<Domain> = SdoEntry::new(0x4211, 0x49, "Bike 73");
    /// `Bike 74` — rw
    pub const BIKE_74: SdoEntry<Domain> = SdoEntry::new(0x4211, 0x4A, "Bike 74");
    /// `Bike 75` — rw
    pub const BIKE_75: SdoEntry<Domain> = SdoEntry::new(0x4211, 0x4B, "Bike 75");
    /// `Bike 76` — rw
    pub const BIKE_76: SdoEntry<Domain> = SdoEntry::new(0x4211, 0x4C, "Bike 76");
    /// `Bike 77` — rw
    pub const BIKE_77: SdoEntry<Domain> = SdoEntry::new(0x4211, 0x4D, "Bike 77");
    /// `Bike 78` — rw
    pub const BIKE_78: SdoEntry<Domain> = SdoEntry::new(0x4211, 0x4E, "Bike 78");
    /// `Bike 79` — rw
    pub const BIKE_79: SdoEntry<Domain> = SdoEntry::new(0x4211, 0x4F, "Bike 79");
    /// `Bike 80` — rw
    pub const BIKE_80: SdoEntry<Domain> = SdoEntry::new(0x4211, 0x50, "Bike 80");
    /// `Bike 81` — rw
    pub const BIKE_81: SdoEntry<Domain> = SdoEntry::new(0x4211, 0x51, "Bike 81");
    /// `Bike 82` — rw
    pub const BIKE_82: SdoEntry<Domain> = SdoEntry::new(0x4211, 0x52, "Bike 82");
    /// `Bike 83` — rw
    pub const BIKE_83: SdoEntry<Domain> = SdoEntry::new(0x4211, 0x53, "Bike 83");
    /// `Bike 84` — rw
    pub const BIKE_84: SdoEntry<Domain> = SdoEntry::new(0x4211, 0x54, "Bike 84");
    /// `Bike 85` — rw
    pub const BIKE_85: SdoEntry<Domain> = SdoEntry::new(0x4211, 0x55, "Bike 85");
    /// `Bike 86` — rw
    pub const BIKE_86: SdoEntry<Domain> = SdoEntry::new(0x4211, 0x56, "Bike 86");
    /// `Bike 87` — rw
    pub const BIKE_87: SdoEntry<Domain> = SdoEntry::new(0x4211, 0x57, "Bike 87");
    /// `Bike 88` — rw
    pub const BIKE_88: SdoEntry<Domain> = SdoEntry::new(0x4211, 0x58, "Bike 88");
    /// `Bike 89` — rw
    pub const BIKE_89: SdoEntry<Domain> = SdoEntry::new(0x4211, 0x59, "Bike 89");
    /// `Bike 90` — rw
    pub const BIKE_90: SdoEntry<Domain> = SdoEntry::new(0x4211, 0x5A, "Bike 90");
    /// `Bike 91` — rw
    pub const BIKE_91: SdoEntry<Domain> = SdoEntry::new(0x4211, 0x5B, "Bike 91");
    /// `Bike 92` — rw
    pub const BIKE_92: SdoEntry<Domain> = SdoEntry::new(0x4211, 0x5C, "Bike 92");
    /// `Bike 93` — rw
    pub const BIKE_93: SdoEntry<Domain> = SdoEntry::new(0x4211, 0x5D, "Bike 93");
    /// `Bike 94` — rw
    pub const BIKE_94: SdoEntry<Domain> = SdoEntry::new(0x4211, 0x5E, "Bike 94");
    /// `Bike 95` — rw
    pub const BIKE_95: SdoEntry<Domain> = SdoEntry::new(0x4211, 0x5F, "Bike 95");
    /// `Bike 96` — rw
    pub const BIKE_96: SdoEntry<Domain> = SdoEntry::new(0x4211, 0x60, "Bike 96");
    /// `Bike 97` — rw
    pub const BIKE_97: SdoEntry<Domain> = SdoEntry::new(0x4211, 0x61, "Bike 97");
    /// `Bike 98` — rw
    pub const BIKE_98: SdoEntry<Domain> = SdoEntry::new(0x4211, 0x62, "Bike 98");
    /// `Bike 99` — rw
    pub const BIKE_99: SdoEntry<Domain> = SdoEntry::new(0x4211, 0x63, "Bike 99");
    /// `Bike 100` — rw
    pub const BIKE_100: SdoEntry<Domain> = SdoEntry::new(0x4211, 0x64, "Bike 100");
}
/// `Account uid` — rw
pub const ACCOUNT_UID: SdoEntry<Domain> = SdoEntry::new(0x4212, 0x00, "Account uid");
/// `Smartphone uid` — rw
pub const SMARTPHONE_UID: SdoEntry<Domain> = SdoEntry::new(0x4213, 0x00, "Smartphone uid");
/// `Token Nounce` — rw
pub const TOKEN_NOUNCE: SdoEntry<Domain> = SdoEntry::new(0x4214, 0x00, "Token Nounce");
/// `Charger validity` — rw, default `0`
pub const CHARGER_VALIDITY: SdoEntry<i64> = SdoEntry::new(0x4219, 0x00, "Charger validity");
/// `Battery error` — ro, default `0`
pub const BATTERY_ERROR: SdoEntry<u64> = SdoEntry::new(0x4221, 0x00, "Battery error");
/// `Charger error` — rw, default `0`
pub const CHARGER_ERROR: SdoEntry<u32> = SdoEntry::new(0x4222, 0x00, "Charger error");
/// `Bike error` — wo, default `0`
pub const BIKE_ERROR: SdoEntry<u64> = SdoEntry::new(0x4223, 0x00, "Bike error");
/// `Battery error history` (0x4224, RECORD)
pub mod battery_error_history {
    use ::canopen_async::dict::*;
    /// `Highest sub-index supported` — ro, default `0x02`
    pub const HIGHEST_SUB_INDEX_SUPPORTED: SdoEntry<u8> = SdoEntry::new(0x4224, 0x00, "Highest sub-index supported");
    /// `Total number` — ro, default `0`
    pub const TOTAL_NUMBER: SdoEntry<u32> = SdoEntry::new(0x4224, 0x01, "Total number");
    /// `History` — ro
    pub const HISTORY: SdoEntry<Domain> = SdoEntry::new(0x4224, 0x02, "History");
}
/// `Battery fixed parameters` (0x4231, RECORD)
pub mod battery_fixed_parameters {
    use ::canopen_async::dict::*;
    /// `Highest sub-index supported` — ro, default `0x0D`
    pub const HIGHEST_SUB_INDEX_SUPPORTED: SdoEntry<u8> = SdoEntry::new(0x4231, 0x00, "Highest sub-index supported");
    /// `rated capacity` — ro, default `0xFFFF`
    pub const RATED_CAPACITY: SdoEntry<u16> = SdoEntry::new(0x4231, 0x01, "rated capacity");
    /// `typical capacity` — ro, default `0xFFFF`
    pub const TYPICAL_CAPACITY: SdoEntry<u16> = SdoEntry::new(0x4231, 0x02, "typical capacity");
    /// `nominal voltage` — ro, default `0xFFFF`
    pub const NOMINAL_VOLTAGE: SdoEntry<u32> = SdoEntry::new(0x4231, 0x03, "nominal voltage");
    /// `charging voltage` — ro, default `0xFFFF`
    pub const CHARGING_VOLTAGE: SdoEntry<u16> = SdoEntry::new(0x4231, 0x04, "charging voltage");
    /// `standard charge` — ro, default `0xFFFF`
    pub const STANDARD_CHARGE: SdoEntry<u16> = SdoEntry::new(0x4231, 0x05, "standard charge");
    /// `standard discharge` — ro, default `0xFFFF`
    pub const STANDARD_DISCHARGE: SdoEntry<u16> = SdoEntry::new(0x4231, 0x06, "standard discharge");
    /// `max charge current` — ro, default `0xFFFF`
    pub const MAX_CHARGE_CURRENT: SdoEntry<u16> = SdoEntry::new(0x4231, 0x07, "max charge current");
    /// `max discharge current` — ro, default `0xFFFF`
    pub const MAX_DISCHARGE_CURRENT: SdoEntry<u16> = SdoEntry::new(0x4231, 0x08, "max discharge current");
    /// `min operating discharge temperature` — ro, default `0xFF`
    pub const MIN_OPERATING_DISCHARGE_TEMPERATURE: SdoEntry<i8> = SdoEntry::new(0x4231, 0x09, "min operating discharge temperature");
    /// `max operating discharge temperature` — ro, default `0xFF`
    pub const MAX_OPERATING_DISCHARGE_TEMPERATURE: SdoEntry<i8> = SdoEntry::new(0x4231, 0x0A, "max operating discharge temperature");
    /// `min operating charge temperature` — ro, default `0xFF`
    pub const MIN_OPERATING_CHARGE_TEMPERATURE: SdoEntry<i8> = SdoEntry::new(0x4231, 0x0B, "min operating charge temperature");
    /// `max operating charge temperature` — ro, default `0xFF`
    pub const MAX_OPERATING_CHARGE_TEMPERATURE: SdoEntry<i8> = SdoEntry::new(0x4231, 0x0C, "max operating charge temperature");
    /// `charge timeout` — ro, default `0xFFFF`
    pub const CHARGE_TIMEOUT: SdoEntry<u16> = SdoEntry::new(0x4231, 0x0D, "charge timeout");
}
/// `BMS information` (0x4233, RECORD)
pub mod bms_information {
    use ::canopen_async::dict::*;
    /// `Highest sub-index supported` — ro, default `0x07`
    pub const HIGHEST_SUB_INDEX_SUPPORTED: SdoEntry<u8> = SdoEntry::new(0x4233, 0x00, "Highest sub-index supported");
    /// `software version` — ro, default `0`
    pub const SOFTWARE_VERSION: SdoEntry<u16> = SdoEntry::new(0x4233, 0x01, "software version");
    /// `hardware version` — ro, default `0`
    pub const HARDWARE_VERSION: SdoEntry<u16> = SdoEntry::new(0x4233, 0x02, "hardware version");
    /// `number of cells in parallel` — ro, default `0`
    pub const NUMBER_OF_CELLS_IN_PARALLEL: SdoEntry<u8> = SdoEntry::new(0x4233, 0x03, "number of cells in parallel");
    /// `number of cells in series` — ro, default `0`
    pub const NUMBER_OF_CELLS_IN_SERIES: SdoEntry<u8> = SdoEntry::new(0x4233, 0x04, "number of cells in series");
    /// `cells chemistry` — ro, default `0`
    pub const CELLS_CHEMISTRY: SdoEntry<u16> = SdoEntry::new(0x4233, 0x05, "cells chemistry");
    /// `cells model` — ro, default `0`
    pub const CELLS_MODEL: SdoEntry<u16> = SdoEntry::new(0x4233, 0x06, "cells model");
    /// `manufacturer name` — ro, default `0`
    pub const MANUFACTURER_NAME: SdoEntry<u32> = SdoEntry::new(0x4233, 0x07, "manufacturer name");
}
/// `Battery UID` — ro, default `000000000000`
pub const BATTERY_UID: SdoEntry<VisibleString> = SdoEntry::new(0x4234, 0x00, "Battery UID");
/// `Battery validity` — rw, default `0`
pub const BATTERY_VALIDITY: SdoEntry<i64> = SdoEntry::new(0x4235, 0x00, "Battery validity");
/// `Battery information` (0x4236, RECORD)
pub mod battery_information {
    use ::canopen_async::dict::*;
    /// `Highest sub-index supported` — ro, default `0x02`
    pub const HIGHEST_SUB_INDEX_SUPPORTED: SdoEntry<u8> = SdoEntry::new(0x4236, 0x00, "Highest sub-index supported");
    /// `hardware version` — rw, default `0`
    pub const HARDWARE_VERSION: SdoEntry<u32> = SdoEntry::new(0x4236, 0x01, "hardware version");
    /// `software version` — rw, default `0`
    pub const SOFTWARE_VERSION: SdoEntry<u32> = SdoEntry::new(0x4236, 0x02, "software version");
}
/// `State` (0x4241, RECORD)
pub mod state {
    use ::canopen_async::dict::*;
    /// `Highest sub-index supported` — ro, default `0x0B`
    pub const HIGHEST_SUB_INDEX_SUPPORTED: SdoEntry<u8> = SdoEntry::new(0x4241, 0x00, "Highest sub-index supported");
    /// `State of charge` — ro, default `0`
    pub const STATE_OF_CHARGE: SdoEntry<u8> = SdoEntry::new(0x4241, 0x01, "State of charge");
    /// `State of health` — ro, default `0`
    pub const STATE_OF_HEALTH: SdoEntry<u8> = SdoEntry::new(0x4241, 0x02, "State of health");
    /// `Cycle count` — ro, default `0`
    pub const CYCLE_COUNT: SdoEntry<u16> = SdoEntry::new(0x4241, 0x03, "Cycle count");
    /// `Current` — ro, default `0`
    pub const CURRENT: SdoEntry<i32> = SdoEntry::new(0x4241, 0x04, "Current");
    /// `Pack voltage` — ro, default `0`
    pub const PACK_VOLTAGE: SdoEntry<u16> = SdoEntry::new(0x4241, 0x05, "Pack voltage");
    /// `Max temperature` — ro, default `0`
    pub const MAX_TEMPERATURE: SdoEntry<i8> = SdoEntry::new(0x4241, 0x06, "Max temperature");
    /// `Min temperature` — ro, default `0`
    pub const MIN_TEMPERATURE: SdoEntry<i8> = SdoEntry::new(0x4241, 0x07, "Min temperature");
    /// `Status` — ro, default `0`
    pub const STATUS: SdoEntry<u16> = SdoEntry::new(0x4241, 0x08, "Status");
    /// `Lock Status` — ro, default `0`
    pub const LOCK_STATUS: SdoEntry<u8> = SdoEntry::new(0x4241, 0x09, "Lock Status");
    /// `Time` — ro, default `0`
    pub const TIME: SdoEntry<u64> = SdoEntry::new(0x4241, 0x0A, "Time");
    /// `Remaining capacity` — ro, default `0`
    pub const REMAINING_CAPACITY: SdoEntry<u16> = SdoEntry::new(0x4241, 0x0B, "Remaining capacity");
}
/// `Cells voltage` (0x4242, ARRAY): 13 × u16
pub mod cells_voltage {
    use ::canopen_async::dict::*;
    pub const ENTRIES: SdoArray<u16> = SdoArray::new(0x4242, 13, "Cells voltage");
    /// `Sub Object 1` — ro, default `0`
    pub const SUB_OBJECT_1: SdoEntry<u16> = SdoEntry::new(0x4242, 0x01, "Sub Object 1");
    /// `Sub Object 2` — ro, default `0`
    pub const SUB_OBJECT_2: SdoEntry<u16> = SdoEntry::new(0x4242, 0x02, "Sub Object 2");
    /// `Sub Object 3` — ro, default `0`
    pub const SUB_OBJECT_3: SdoEntry<u16> = SdoEntry::new(0x4242, 0x03, "Sub Object 3");
    /// `Sub Object 4` — ro, default `0`
    pub const SUB_OBJECT_4: SdoEntry<u16> = SdoEntry::new(0x4242, 0x04, "Sub Object 4");
    /// `Sub Object 5` — ro, default `0`
    pub const SUB_OBJECT_5: SdoEntry<u16> = SdoEntry::new(0x4242, 0x05, "Sub Object 5");
    /// `Sub Object 6` — ro, default `0`
    pub const SUB_OBJECT_6: SdoEntry<u16> = SdoEntry::new(0x4242, 0x06, "Sub Object 6");
    /// `Sub Object 7` — ro, default `0`
    pub const SUB_OBJECT_7: SdoEntry<u16> = SdoEntry::new(0x4242, 0x07, "Sub Object 7");
    /// `Sub Object 8` — ro, default `0`
    pub const SUB_OBJECT_8: SdoEntry<u16> = SdoEntry::new(0x4242, 0x08, "Sub Object 8");
    /// `Sub Object 9` — ro, default `0`
    pub const SUB_OBJECT_9: SdoEntry<u16> = SdoEntry::new(0x4242, 0x09, "Sub Object 9");
    /// `Sub Object 10` — ro, default `0`
    pub const SUB_OBJECT_10: SdoEntry<u16> = SdoEntry::new(0x4242, 0x0A, "Sub Object 10");
    /// `Sub Object 11` — ro, default `0`
    pub const SUB_OBJECT_11: SdoEntry<u16> = SdoEntry::new(0x4242, 0x0B, "Sub Object 11");
    /// `Sub Object 12` — ro, default `0`
    pub const SUB_OBJECT_12: SdoEntry<u16> = SdoEntry::new(0x4242, 0x0C, "Sub Object 12");
    /// `Sub Object 13` — ro, default `0`
    pub const SUB_OBJECT_13: SdoEntry<u16> = SdoEntry::new(0x4242, 0x0D, "Sub Object 13");
}
/// `Cells temperature` (0x4243, ARRAY): 4 × i8
pub mod cells_temperature {
    use ::canopen_async::dict::*;
    pub const ENTRIES: SdoArray<i8> = SdoArray::new(0x4243, 4, "Cells temperature");
    /// `Sub Object 1` — rw, default `0`
    pub const SUB_OBJECT_1: SdoEntry<i8> = SdoEntry::new(0x4243, 0x01, "Sub Object 1");
    /// `Sub Object 2` — rw, default `0`
    pub const SUB_OBJECT_2: SdoEntry<i8> = SdoEntry::new(0x4243, 0x02, "Sub Object 2");
    /// `Sub Object 3` — rw, default `0`
    pub const SUB_OBJECT_3: SdoEntry<i8> = SdoEntry::new(0x4243, 0x03, "Sub Object 3");
    /// `Sub Object 4` — rw, default `0`
    pub const SUB_OBJECT_4: SdoEntry<i8> = SdoEntry::new(0x4243, 0x04, "Sub Object 4");
}
/// `Bike Info` (0x4249, RECORD)
pub mod bike_info {
    use ::canopen_async::dict::*;
    /// `Highest sub-index supported` — ro, default `0x03`
    pub const HIGHEST_SUB_INDEX_SUPPORTED: SdoEntry<u8> = SdoEntry::new(0x4249, 0x00, "Highest sub-index supported");
    /// `HW Version` — rw, default `0`
    pub const HW_VERSION: SdoEntry<u32> = SdoEntry::new(0x4249, 0x01, "HW Version");
    /// `SW Version` — rw, default `0`
    pub const SW_VERSION: SdoEntry<u32> = SdoEntry::new(0x4249, 0x02, "SW Version");
    /// `Bike type` — rw, default `0`
    pub const BIKE_TYPE: SdoEntry<u8> = SdoEntry::new(0x4249, 0x03, "Bike type");
}
/// `Bike State` (0x4250, RECORD)
pub mod bike_state {
    use ::canopen_async::dict::*;
    /// `Highest sub-index supported` — ro, default `0x07`
    pub const HIGHEST_SUB_INDEX_SUPPORTED: SdoEntry<u8> = SdoEntry::new(0x4250, 0x00, "Highest sub-index supported");
    /// `Speed` — rw, default `0`
    pub const SPEED: SdoEntry<u16> = SdoEntry::new(0x4250, 0x01, "Speed");
    /// `Average Speed` — rw, default `0`
    pub const AVERAGE_SPEED: SdoEntry<u16> = SdoEntry::new(0x4250, 0x02, "Average Speed");
    /// `Max Speed` — rw, default `0`
    pub const MAX_SPEED: SdoEntry<u16> = SdoEntry::new(0x4250, 0x03, "Max Speed");
    /// `Trip distance` — rw, default `0`
    pub const TRIP_DISTANCE: SdoEntry<u16> = SdoEntry::new(0x4250, 0x04, "Trip distance");
    /// `remaining mileage` — rw, default `0`
    pub const REMAINING_MILEAGE: SdoEntry<u16> = SdoEntry::new(0x4250, 0x05, "remaining mileage");
    /// `total mileage` — rw, default `0`
    pub const TOTAL_MILEAGE: SdoEntry<u32> = SdoEntry::new(0x4250, 0x06, "total mileage");
    /// `Maintenance mileage` — rw, default `0`
    pub const MAINTENANCE_MILEAGE: SdoEntry<u32> = SdoEntry::new(0x4250, 0x07, "Maintenance mileage");
}
/// `Secondary` (0x4255, RECORD)
pub mod secondary {
    use ::canopen_async::dict::*;
    /// `Highest sub-index supported` — ro, default `0x0A`
    pub const HIGHEST_SUB_INDEX_SUPPORTED: SdoEntry<u8> = SdoEntry::new(0x4255, 0x00, "Highest sub-index supported");
    /// `Hardware version` — wo, default `0`
    pub const HARDWARE_VERSION: SdoEntry<u32> = SdoEntry::new(0x4255, 0x01, "Hardware version");
    /// `Software version` — wo, default `0`
    pub const SOFTWARE_VERSION: SdoEntry<u32> = SdoEntry::new(0x4255, 0x02, "Software version");
    /// `SOC` — wo, default `0`
    pub const SOC: SdoEntry<u8> = SdoEntry::new(0x4255, 0x03, "SOC");
    /// `SOH` — wo, default `0`
    pub const SOH: SdoEntry<u8> = SdoEntry::new(0x4255, 0x04, "SOH");
    /// `Max temperature` — wo, default `0`
    pub const MAX_TEMPERATURE: SdoEntry<i8> = SdoEntry::new(0x4255, 0x05, "Max temperature");
    /// `Error` — wo, default `0`
    pub const ERROR: SdoEntry<u64> = SdoEntry::new(0x4255, 0x06, "Error");
    /// `Status` — wo, default `0`
    pub const STATUS: SdoEntry<u16> = SdoEntry::new(0x4255, 0x07, "Status");
    /// `Lock status` — wo, default `0`
    pub const LOCK_STATUS: SdoEntry<u8> = SdoEntry::new(0x4255, 0x08, "Lock status");
    /// `Time` — wo, default `0`
    pub const TIME: SdoEntry<u64> = SdoEntry::new(0x4255, 0x09, "Time");
    /// `UID` — wo, default `000000000000`
    pub const UID: SdoEntry<VisibleString> = SdoEntry::new(0x4255, 0x0A, "UID");
}
/// `Battery OD version` — ro, default `2319`
pub const BATTERY_OD_VERSION: SdoEntry<u32> = SdoEntry::new(0x4260, 0x00, "Battery OD version");
/// `Bike OD version` — wo, default `2319`
pub const BIKE_OD_VERSION: SdoEntry<u32> = SdoEntry::new(0x4261, 0x00, "Bike OD version");
/// `Nonce Signature` — ro
pub const NONCE_SIGNATURE: SdoEntry<Domain> = SdoEntry::new(0x4262, 0x00, "Nonce Signature");
/// `Bike Id Signature` — rw
pub const BIKE_ID_SIGNATURE: SdoEntry<Domain> = SdoEntry::new(0x4263, 0x00, "Bike Id Signature");
/// `Station Auth` (0x4264, ARRAY): 2 × Domain
pub mod station_auth {
    use ::canopen_async::dict::*;
    pub const ENTRIES: SdoArray<Domain> = SdoArray::new(0x4264, 2, "Station Auth");
    /// `Nonce` — rw
    pub const NONCE: SdoEntry<Domain> = SdoEntry::new(0x4264, 0x01, "Nonce");
    /// `Signature` — rw
    pub const SIGNATURE: SdoEntry<Domain> = SdoEntry::new(0x4264, 0x02, "Signature");
}
/// `Status Word Auth` (0x4265, ARRAY): 2 × Domain
pub mod status_word_auth {
    use ::canopen_async::dict::*;
    pub const ENTRIES: SdoArray<Domain> = SdoArray::new(0x4265, 2, "Status Word Auth");
    /// `Nonce` — rw
    pub const NONCE: SdoEntry<Domain> = SdoEntry::new(0x4265, 0x01, "Nonce");
    /// `Signature` — rw
    pub const SIGNATURE: SdoEntry<Domain> = SdoEntry::new(0x4265, 0x02, "Signature");
}
/// `Control Word Auth` (0x4266, ARRAY): 2 × Domain
pub mod control_word_auth {
    use ::canopen_async::dict::*;
    pub const ENTRIES: SdoArray<Domain> = SdoArray::new(0x4266, 2, "Control Word Auth");
    /// `Nonce` — rw
    pub const NONCE: SdoEntry<Domain> = SdoEntry::new(0x4266, 0x01, "Nonce");
    /// `Signature` — rw
    pub const SIGNATURE: SdoEntry<Domain> = SdoEntry::new(0x4266, 0x02, "Signature");
}
/// `BMS Part Number` — rw
pub const BMS_PART_NUMBER: SdoEntry<Domain> = SdoEntry::new(0x4267, 0x00, "BMS Part Number");
