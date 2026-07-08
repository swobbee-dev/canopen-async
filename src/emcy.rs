//! Emergency object (EMCY) parsing (CiA 301, 7.2.7).

/// A received emergency message.
///
/// CiA 301 specifies 8 data bytes; some devices (e.g. the lime battery BMS)
/// send fewer. Parsing is length-tolerant: missing tail bytes read as zero.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct EmcyMessage {
    /// Emergency error code (e.g. 0x0000 = error reset, 0x1000 = generic)
    pub error_code: u16,
    /// Value of the error register (object 0x1001)
    pub error_register: u8,
    /// Manufacturer-specific error field
    pub vendor: [u8; 5],
}

impl EmcyMessage {
    /// Parse an EMCY payload of any length; missing bytes are zero.
    pub fn parse(data: &[u8]) -> Self {
        let mut padded = [0u8; 8];
        let len = data.len().min(8);
        padded[..len].copy_from_slice(&data[..len]);

        let mut vendor = [0u8; 5];
        vendor.copy_from_slice(&padded[3..8]);

        EmcyMessage {
            error_code: u16::from_le_bytes([padded[0], padded[1]]),
            error_register: padded[2],
            vendor,
        }
    }

    /// An error code of 0 with error register 0 signals "error reset / no error".
    pub fn is_error_reset(&self) -> bool {
        self.error_code == 0 && self.error_register == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_standard_8_byte_frame() {
        let msg = EmcyMessage::parse(&[0x01, 0x10, 0x81, 1, 2, 3, 4, 5]);
        assert_eq!(msg.error_code, 0x1001);
        assert_eq!(msg.error_register, 0x81);
        assert_eq!(msg.vendor, [1, 2, 3, 4, 5]);
        assert!(!msg.is_error_reset());
    }

    #[test]
    fn parse_short_lime_frame() {
        // The lime battery sends 3-byte EMCY frames
        let msg = EmcyMessage::parse(&[0xFF, 0x10, 0x20]);
        assert_eq!(msg.error_code, 0x10FF);
        assert_eq!(msg.error_register, 0x20);
        assert_eq!(msg.vendor, [0; 5]);
    }

    #[test]
    fn parse_error_reset() {
        let msg = EmcyMessage::parse(&[0, 0, 0, 0, 0, 0, 0, 0]);
        assert!(msg.is_error_reset());
    }
}
