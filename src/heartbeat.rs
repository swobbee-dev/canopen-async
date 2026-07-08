//! Heartbeat supervision for one remote node (CiA 301, 7.2.8.2.2).
//!
//! The monitor is fed by [`NodeClient::handle_frame`](crate::NodeClient) and
//! tracks when the node was last heard and in which NMT state, so
//! applications get liveness supervision instead of hand-rolling per-driver
//! signal-and-timeout logic.

use core::cell::Cell;

use embassy_sync::blocking_mutex::Mutex;
use embassy_sync::blocking_mutex::raw::RawMutex;
use embassy_sync::signal::Signal;
use embassy_time::{Duration, Instant, with_timeout};

use crate::frame::NmtState;

#[derive(Debug, Clone, Copy, Default)]
struct Inner {
    last_state: Option<NmtState>,
    last_seen: Option<Instant>,
}

/// Tracks the heartbeat of one node.
pub struct HeartbeatMonitor<M: RawMutex> {
    inner: Mutex<M, Cell<Inner>>,
    beat: Signal<M, NmtState>,
}

impl<M: RawMutex> HeartbeatMonitor<M> {
    pub const fn new() -> Self {
        Self {
            inner: Mutex::new(Cell::new(Inner {
                last_state: None,
                last_seen: None,
            })),
            beat: Signal::new(),
        }
    }

    /// Record a received heartbeat. Called by
    /// [`NodeClient::handle_frame`](crate::NodeClient); synchronous, safe
    /// from any RX context.
    pub(crate) fn on_heartbeat(&self, state: NmtState) {
        self.inner.lock(|inner| {
            inner.set(Inner {
                last_state: Some(state),
                last_seen: Some(Instant::now()),
            });
        });
        self.beat.signal(state);
    }

    /// The NMT state reported by the most recent heartbeat, if any was
    /// received since boot.
    pub fn last_state(&self) -> Option<NmtState> {
        self.inner.lock(|inner| inner.get().last_state)
    }

    /// When the last heartbeat (or boot-up message) was received.
    pub fn last_seen(&self) -> Option<Instant> {
        self.inner.lock(|inner| inner.get().last_seen)
    }

    /// Whether a heartbeat was received within the last `max_age`.
    pub fn is_alive(&self, max_age: Duration) -> bool {
        self.last_seen()
            .is_some_and(|seen| Instant::now() - seen <= max_age)
    }

    /// Wait for the next heartbeat received *after* this call and return the
    /// reported NMT state. A previously stored beat is discarded first.
    pub async fn wait_beat(&self) -> NmtState {
        self.beat.reset();
        self.beat.wait().await
    }

    /// Wait for a fresh heartbeat with a timeout; `None` on timeout.
    pub async fn wait_beat_timeout(&self, timeout: Duration) -> Option<NmtState> {
        with_timeout(timeout, self.wait_beat()).await.ok()
    }

    /// Resolve when the node goes silent: no heartbeat for `timeout` (which
    /// callers derive from the node's producer period plus margin). Intended
    /// for a supervision task; returns the last known state, if any.
    pub async fn wait_lost(&self, timeout: Duration) -> Option<NmtState> {
        loop {
            if self.wait_beat_timeout(timeout).await.is_none() {
                return self.last_state();
            }
        }
    }
}

impl<M: RawMutex> Default for HeartbeatMonitor<M> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
    use futures::FutureExt;
    use futures::executor::block_on;

    type Monitor = HeartbeatMonitor<CriticalSectionRawMutex>;

    #[test]
    fn tracks_last_state_and_liveness() {
        let mon = Monitor::new();
        assert_eq!(mon.last_state(), None);
        assert!(!mon.is_alive(Duration::from_secs(1)));

        mon.on_heartbeat(NmtState::Operational);
        assert_eq!(mon.last_state(), Some(NmtState::Operational));
        assert!(mon.is_alive(Duration::from_secs(1)));

        mon.on_heartbeat(NmtState::PreOperational);
        assert_eq!(mon.last_state(), Some(NmtState::PreOperational));
    }

    #[test]
    fn wait_beat_discards_stale_beat() {
        let mon = Monitor::new();
        mon.on_heartbeat(NmtState::Operational); // stale

        let mut fut = core::pin::pin!(mon.wait_beat());
        assert!(fut.as_mut().now_or_never().is_none());

        mon.on_heartbeat(NmtState::Stopped);
        assert_eq!(fut.now_or_never(), Some(NmtState::Stopped));
    }

    #[test]
    fn wait_beat_timeout_expires_without_beats() {
        let mon = Monitor::new();
        let result = block_on(mon.wait_beat_timeout(Duration::from_millis(20)));
        assert_eq!(result, None);
    }

    #[test]
    fn wait_lost_survives_beats_then_fires() {
        let mon = Monitor::new();
        mon.on_heartbeat(NmtState::Operational);

        let lost = block_on(async {
            futures::join!(
                mon.wait_lost(Duration::from_millis(30)),
                async {
                    // One more beat inside the window, then silence
                    embassy_time::Timer::after(Duration::from_millis(10)).await;
                    mon.on_heartbeat(NmtState::Operational);
                }
            )
            .0
        });
        assert_eq!(lost, Some(NmtState::Operational));
    }
}
