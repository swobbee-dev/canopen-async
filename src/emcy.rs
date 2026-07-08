//! Emergency object (EMCY) parsing and per-node monitoring (CiA 301, 7.2.7).

use core::cell::Cell;

use embassy_sync::blocking_mutex::Mutex;
use embassy_sync::blocking_mutex::raw::RawMutex;
use embassy_sync::signal::Signal;

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

/// Tracks emergency messages from one node: latest message, whether an
/// error is currently active (cleared by an "error reset" EMCY), and an
/// async waiter. Fed by [`NodeClient::handle_frame`](crate::NodeClient).
pub struct EmcyMonitor<M: RawMutex> {
    latest: Mutex<M, Cell<Option<EmcyMessage>>>,
    received: Signal<M, EmcyMessage>,
}

impl<M: RawMutex> EmcyMonitor<M> {
    pub const fn new() -> Self {
        Self {
            latest: Mutex::new(Cell::new(None)),
            received: Signal::new(),
        }
    }

    /// Record a received EMCY. Synchronous, safe from any RX context.
    pub(crate) fn on_emcy(&self, msg: EmcyMessage) {
        self.latest.lock(|latest| latest.set(Some(msg)));
        self.received.signal(msg);
    }

    /// The most recent EMCY received, if any.
    pub fn latest(&self) -> Option<EmcyMessage> {
        self.latest.lock(|latest| latest.get())
    }

    /// Whether the node's last EMCY reported an error (and no error reset
    /// has been received since).
    pub fn error_active(&self) -> bool {
        self.latest().is_some_and(|msg| !msg.is_error_reset())
    }

    /// Wait for the next EMCY received *after* this call; a previously
    /// stored message is discarded first.
    pub async fn wait_emcy(&self) -> EmcyMessage {
        self.received.reset();
        self.received.wait().await
    }
}

impl<M: RawMutex> Default for EmcyMonitor<M> {
    fn default() -> Self {
        Self::new()
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

    #[test]
    fn monitor_tracks_active_error_and_reset() {
        use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
        let mon: EmcyMonitor<CriticalSectionRawMutex> = EmcyMonitor::new();
        assert!(!mon.error_active());
        assert_eq!(mon.latest(), None);

        let error = EmcyMessage::parse(&[0x01, 0x10, 0x81, 0, 0, 0, 0, 0]);
        mon.on_emcy(error);
        assert!(mon.error_active());
        assert_eq!(mon.latest(), Some(error));

        // Error reset clears the active flag but stays visible as latest
        let reset = EmcyMessage::parse(&[0, 0, 0, 0, 0, 0, 0, 0]);
        mon.on_emcy(reset);
        assert!(!mon.error_active());
        assert_eq!(mon.latest(), Some(reset));
    }

    #[test]
    fn monitor_wait_discards_stale_message() {
        use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
        use futures::FutureExt;

        let mon: EmcyMonitor<CriticalSectionRawMutex> = EmcyMonitor::new();
        mon.on_emcy(EmcyMessage::parse(&[0x01, 0x10, 0x81])); // stale

        let mut fut = core::pin::pin!(mon.wait_emcy());
        assert!(fut.as_mut().now_or_never().is_none());

        let fresh = EmcyMessage::parse(&[0x02, 0x10, 0x81]);
        mon.on_emcy(fresh);
        assert_eq!(fut.now_or_never(), Some(fresh));
    }
}
