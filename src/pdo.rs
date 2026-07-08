//! Typed reception of PDOs transmitted by a node.
//!
//! PDO payload layouts are device-specific (fixed by the vendor's PDO
//! mapping), so applications define one struct per PDO and implement
//! [`PdoPayload`] for it — plain byte-slice decoding, typically a handful of
//! `u16::from_le_bytes` lines. A [`PdoSlot`] is the hand-off point between
//! the CAN RX context and consumers: latest-value semantics, one waiter.
//!
//! ```ignore
//! struct Pdo1 { voltage_mv: u16, current_ma: i32 }
//!
//! impl PdoPayload for Pdo1 {
//!     fn decode(d: &[u8]) -> Option<Self> {
//!         let d: &[u8; 8] = d.try_into().ok()?;
//!         Some(Self {
//!             voltage_mv: u16::from_le_bytes([d[2], d[3]]),
//!             current_ma: i32::from_le_bytes([d[4], d[5], d[6], d[7]]),
//!         })
//!     }
//! }
//!
//! static PDO1: PdoSlot<CriticalSectionRawMutex, Pdo1> = PdoSlot::new();
//!
//! // RX context:
//! if let Some(NodeEvent::Tpdo { num: 1, data }) = client.handle_frame(&frame) {
//!     PDO1.publish_raw(data);
//! }
//! // Consumer:
//! let pdo1 = PDO1.wait_fresh().await;
//! ```

use embassy_sync::blocking_mutex::raw::RawMutex;
use embassy_sync::signal::Signal;

/// Decode a PDO payload into a typed value.
pub trait PdoPayload: Sized {
    /// Decode from the PDO data bytes; `None` if the payload is too short or
    /// otherwise invalid (the frame is then dropped).
    fn decode(data: &[u8]) -> Option<Self>;
}

/// Latest-value cell carrying one decoded PDO from the RX context to a
/// consumer. A newer PDO overwrites an unconsumed older one, which is the
/// correct semantic for periodic process data.
pub struct PdoSlot<M: RawMutex, T: PdoPayload + Send> {
    signal: Signal<M, T>,
}

impl<M: RawMutex, T: PdoPayload + Send> PdoSlot<M, T> {
    pub const fn new() -> Self {
        Self { signal: Signal::new() }
    }

    /// Decode and publish a received payload. Payloads `T` fails to decode
    /// are dropped.
    pub fn publish_raw(&self, data: &[u8]) {
        if let Some(value) = T::decode(data) {
            self.signal.signal(value);
        }
    }

    /// Wait for the next PDO received *after* this call: any previously
    /// stored value is discarded first.
    pub async fn wait_fresh(&self) -> T {
        self.signal.reset();
        self.signal.wait().await
    }

    /// Take the most recently received value, if any.
    pub fn try_latest(&self) -> Option<T> {
        self.signal.try_take()
    }
}

impl<M: RawMutex, T: PdoPayload + Send> Default for PdoSlot<M, T> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use embassy_sync::blocking_mutex::raw::NoopRawMutex;
    use futures::FutureExt;

    #[derive(Debug, PartialEq)]
    struct TestPdo {
        value: u16,
    }

    impl PdoPayload for TestPdo {
        fn decode(data: &[u8]) -> Option<Self> {
            let bytes: [u8; 2] = data.get(..2)?.try_into().ok()?;
            Some(TestPdo { value: u16::from_le_bytes(bytes) })
        }
    }

    #[test]
    fn latest_value_semantics() {
        let slot: PdoSlot<NoopRawMutex, TestPdo> = PdoSlot::new();
        assert_eq!(slot.try_latest(), None);

        slot.publish_raw(&[0x34, 0x12]);
        slot.publish_raw(&[0x78, 0x56]); // overwrites
        assert_eq!(slot.try_latest(), Some(TestPdo { value: 0x5678 }));
        assert_eq!(slot.try_latest(), None); // consumed
    }

    #[test]
    fn undecodable_payload_is_dropped() {
        let slot: PdoSlot<NoopRawMutex, TestPdo> = PdoSlot::new();
        slot.publish_raw(&[0x01]); // too short
        assert_eq!(slot.try_latest(), None);
    }

    #[test]
    fn wait_fresh_discards_stale_value() {
        let slot: PdoSlot<NoopRawMutex, TestPdo> = PdoSlot::new();
        slot.publish_raw(&[0x34, 0x12]); // stale

        let mut fut = core::pin::pin!(slot.wait_fresh());
        // The stale value must not satisfy the wait
        assert!(fut.as_mut().now_or_never().is_none());

        slot.publish_raw(&[0x78, 0x56]);
        assert_eq!(fut.now_or_never(), Some(TestPdo { value: 0x5678 }));
    }
}
