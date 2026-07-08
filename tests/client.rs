//! Client-level tests driving `SdoClient` through its public API with a mock
//! CAN transport. Responses are injected via `process_frame`, exactly as an
//! application RX task would.

mod common;

use core::cell::RefCell;
use std::rc::Rc;
use std::vec::Vec;

use canopen_async::{SdoClient, SdoError};
use common::{MockTx, TestFrame};
use embassy_time::Duration;
use embedded_can::{Frame, Id, StandardId};
use futures::executor::block_on;
use futures::join;

const NODE_ID: u8 = 0x11;
const SDO_TX_ID: u16 = 0x580 + NODE_ID as u16; // server -> client
const SDO_RX_ID: u16 = 0x600 + NODE_ID as u16; // client -> server

const IDX: u16 = 0x2000;
const SUB: u8 = 0x01;

struct Harness {
    client: SdoClient<TestFrame, MockTx>,
    sent: Rc<RefCell<Vec<TestFrame>>>,
}

impl Harness {
    fn new() -> Self {
        let tx = MockTx::default();
        let sent = tx.sent.clone();
        let client = SdoClient::new(NODE_ID, tx, Duration::from_millis(100)).unwrap();
        Harness { client, sent }
    }

    /// Server response frame with the standard test multiplexer.
    fn response(cs: u8, tail: [u8; 4]) -> TestFrame {
        let idx = IDX.to_le_bytes();
        let payload = [cs, idx[0], idx[1], SUB, tail[0], tail[1], tail[2], tail[3]];
        TestFrame::new(StandardId::new(SDO_TX_ID).unwrap(), &payload).unwrap()
    }

    /// Server response frame with a raw payload (no multiplexer).
    fn raw_response(payload: &[u8]) -> TestFrame {
        TestFrame::new(StandardId::new(SDO_TX_ID).unwrap(), payload).unwrap()
    }

    fn sent_payloads(&self) -> Vec<Vec<u8>> {
        self.sent
            .borrow()
            .iter()
            .map(|f| {
                assert_eq!(f.id, Id::Standard(StandardId::new(SDO_RX_ID).unwrap()));
                f.data.clone()
            })
            .collect()
    }
}

/// Drive a client future while feeding it one batch of response frames per
/// client transmission (a block-upload server sends whole sub-blocks without
/// intervening client frames).
fn run_with_batches<F>(harness: &Harness, fut: F, batches: Vec<Vec<TestFrame>>) -> F::Output
where
    F: core::future::Future,
{
    let feeder = async {
        let mut batches = batches.into_iter().peekable();
        let mut seen_tx = 0;
        // Feed one batch per client transmission; done when all are fed.
        while batches.peek().is_some() {
            let tx_count = harness.sent.borrow().len();
            if tx_count > seen_tx {
                seen_tx = tx_count;
                for resp in batches.next().unwrap() {
                    harness.client.process_frame(&resp);
                }
            } else {
                // Yield so the client future can make progress.
                futures_ext::yield_once().await;
            }
        }
    };
    block_on(async { join!(fut, feeder) }).0
}

/// Drive a client future feeding exactly one response per client transmission.
fn run_with_responses<F>(harness: &Harness, fut: F, responses: Vec<TestFrame>) -> F::Output
where
    F: core::future::Future,
{
    let batches = responses.into_iter().map(|r| vec![r]).collect();
    run_with_batches(harness, fut, batches)
}

// A tiny yield helper; futures 0.3 has no pending_once, so emulate it.
mod futures_ext {
    use core::future::Future;
    use core::pin::Pin;
    use core::task::{Context, Poll};

    pub struct YieldOnce(bool);

    impl Future for YieldOnce {
        type Output = ();
        fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
            if self.0 {
                Poll::Ready(())
            } else {
                self.0 = true;
                cx.waker().wake_by_ref();
                Poll::Pending
            }
        }
    }

    pub fn yield_once() -> YieldOnce {
        YieldOnce(false)
    }
}

#[test]
fn read_expedited_returns_data() {
    let h = Harness::new();
    // 2 valid bytes: cs 0x4B
    let result = run_with_responses(
        &h,
        h.client.read_expedited(IDX, SUB),
        vec![Harness::response(0x4B, [0x34, 0x12, 0, 0])],
    );
    assert_eq!(result.unwrap(), 0x1234);

    // The request must be an initiate upload for the right object
    let sent = h.sent_payloads();
    assert_eq!(sent.len(), 1);
    assert_eq!(sent[0][..4], [0x40, 0x00, 0x20, SUB]);
}

#[test]
fn read_expedited_on_segmented_reply_is_not_expedited() {
    // Regression: a segmented initiate response (0x41) used to be returned
    // as if the announced SIZE were the object VALUE.
    let h = Harness::new();
    let result = run_with_responses(
        &h,
        h.client.read_expedited(IDX, SUB),
        vec![Harness::response(0x41, 260u32.to_le_bytes())],
    );
    assert_eq!(result.unwrap_err(), SdoError::NotExpedited);
    // The server has started a segmented transfer the client won't finish;
    // it must be aborted (0x0800_0000, general error).
    assert_eq!(h.sent_payloads().last().unwrap(), &abort_payload(0x0800_0000));
}

#[test]
fn read_segmented_handles_expedited_reply() {
    // Regression: a server answering expedited to an initiate-upload used to
    // desync read_segmented (data misread as transfer size).
    let h = Harness::new();
    let mut buf = [0u8; 16];
    let result = run_with_responses(
        &h,
        h.client.read_segmented(IDX, SUB, &mut buf),
        vec![Harness::response(0x43, [0xDE, 0xAD, 0xBE, 0xEF])],
    );
    assert_eq!(result.unwrap(), 4);
    assert_eq!(buf[..4], [0xDE, 0xAD, 0xBE, 0xEF]);
    // No upload-segment request may follow the expedited response
    assert_eq!(h.sent_payloads().len(), 1);
}

#[test]
fn read_segmented_full_transfer_returns_length() {
    let h = Harness::new();
    let mut buf = [0u8; 16];
    let result = run_with_responses(
        &h,
        h.client.read_segmented(IDX, SUB, &mut buf),
        vec![
            // Initiate: segmented, 10 bytes total
            Harness::response(0x41, 10u32.to_le_bytes()),
            // Segment 1: toggle 0, 7 bytes, not last
            Harness::raw_response(&[0x00, 1, 2, 3, 4, 5, 6, 7]),
            // Segment 2: toggle 1, 3 bytes (n=4), last -> cs 0001_1001
            Harness::raw_response(&[0x19, 8, 9, 10, 0, 0, 0, 0]),
        ],
    );
    assert_eq!(result.unwrap(), 10);
    assert_eq!(buf[..10], [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
}

#[test]
fn read_expedited_accepts_no_size_response() {
    // cs 0x42: expedited without size indication
    let h = Harness::new();
    let result = run_with_responses(
        &h,
        h.client.read_expedited(IDX, SUB),
        vec![Harness::response(0x42, 0xAABB_CCDDu32.to_le_bytes())],
    );
    assert_eq!(result.unwrap(), 0xAABB_CCDD);
}

/// The abort payload the client must send for `code` on the test object.
fn abort_payload(code: u32) -> Vec<u8> {
    let idx = IDX.to_le_bytes();
    let code = code.to_le_bytes();
    vec![0x80, idx[0], idx[1], SUB, code[0], code[1], code[2], code[3]]
}

#[test]
fn timeout_when_no_response() {
    let h = Harness::new();
    let result = block_on(h.client.read_expedited(IDX, SUB));
    assert_eq!(result.unwrap_err(), SdoError::Timeout);

    // Regression: the client must abort the transfer (0x0504_0000, timeout)
    // so the server releases its state instead of being left mid-transfer.
    let sent = h.sent_payloads();
    assert_eq!(sent.len(), 2);
    assert_eq!(sent[1], abort_payload(0x0504_0000));
}

#[test]
fn invalid_response_sends_abort() {
    let h = Harness::new();
    // cs 0x45 is in the initiate-upload response family but not a valid
    // combination -> InvalidResponse plus an abort (0x0504_0001).
    let result = run_with_responses(
        &h,
        h.client.read_expedited(IDX, SUB),
        vec![Harness::response(0x45, [0; 4])],
    );
    assert_eq!(result.unwrap_err(), SdoError::InvalidResponse);
    assert_eq!(h.sent_payloads().last().unwrap(), &abort_payload(0x0504_0001));
}

#[test]
fn crc_mismatch_sends_abort() {
    let h = Harness::new();
    let mut buf = [0u8; 32];
    let mut batches = block_upload_batches(0xC6, 10u32.to_le_bytes());
    // Corrupt the server CRC in the end frame
    batches.last_mut().unwrap()[0] = Harness::raw_response(&[0xD1, 0xBA, 0xAD, 0, 0, 0, 0, 0]);

    let result = run_with_batches(&h, h.client.read_block(IDX, SUB, &mut buf, true), batches);
    assert_eq!(result.unwrap_err(), SdoError::SdoAbort(0x0504_0004));
    // The client must abort instead of confirming the transfer
    assert_eq!(h.sent_payloads().last().unwrap(), &abort_payload(0x0504_0004));
}

#[test]
fn write_segmented_empty_data_sends_empty_last_segment() {
    // Regression: a zero-length download sent the size-0 initiate and then
    // no segment at all, leaving the server waiting forever.
    let h = Harness::new();
    let result = run_with_responses(
        &h,
        h.client.write_segmented(IDX, SUB, &[]),
        vec![
            Harness::response(0x60, [0; 4]),                     // initiate ack
            Harness::raw_response(&[0x20, 0, 0, 0, 0, 0, 0, 0]), // segment ack, toggle 0
        ],
    );
    assert!(result.is_ok());

    let sent = h.sent_payloads();
    assert_eq!(sent.len(), 2);
    // Empty last segment: cs = 000_0_111_1 (toggle 0, n=7, c=1)
    assert_eq!(sent[1], vec![0x0F, 0, 0, 0, 0, 0, 0, 0]);
}

#[test]
fn stale_frame_does_not_eat_pending_request() {
    // Regression: any frame on the SDO response COB-ID consumed the pending
    // state, so a stale/duplicated frame turned the real response into a
    // timeout. A frame whose command specifier cannot belong to the pending
    // request must be ignored.
    let h = Harness::new();
    let result = run_with_batches(
        &h,
        h.client.read_expedited(IDX, SUB),
        vec![vec![
            // Stale download ack (cs 0x60) — cannot answer an initiate upload
            Harness::response(0x60, [0; 4]),
            // The real expedited response
            Harness::response(0x4B, [0x34, 0x12, 0, 0]),
        ]],
    );
    assert_eq!(result.unwrap(), 0x1234);
}

/// XMODEM CRC over `data`, as used by SDO block transfers.
fn block_crc(data: &[u8]) -> u16 {
    crc::Crc::<u16>::new(&crc::CRC_16_XMODEM).checksum(data)
}

/// Batches for a complete 10-byte block upload: init response (given cs +
/// tail), two segments (7 + 3 bytes), end frame announcing 4 unused bytes.
fn block_upload_batches(init_cs: u8, init_tail: [u8; 4]) -> Vec<Vec<TestFrame>> {
    let crc = block_crc(&[1, 2, 3, 4, 5, 6, 7, 8, 9, 10]).to_le_bytes();
    vec![
        // response to initiate request
        vec![Harness::response(init_cs, init_tail)],
        // response to start-upload: both segments, second with c bit set.
        // The last segment's tail bytes are garbage the client must discard.
        vec![
            Harness::raw_response(&[0x01, 1, 2, 3, 4, 5, 6, 7]),
            Harness::raw_response(&[0x82, 8, 9, 10, 0xEE, 0xEE, 0xEE, 0xEE]),
        ],
        // response to sub-block ack: end frame, n=4 -> cs = 110_100_01
        vec![Harness::raw_response(&[0xD1, crc[0], crc[1], 0, 0, 0, 0, 0])],
    ]
}

#[test]
fn read_block_with_size_returns_length() {
    let h = Harness::new();
    let mut buf = [0u8; 32];
    let result = run_with_batches(
        &h,
        h.client.read_block(IDX, SUB, &mut buf, true),
        block_upload_batches(0xC6, 10u32.to_le_bytes()),
    );
    assert_eq!(result.unwrap(), 10);
    assert_eq!(buf[..10], [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);

    // Client must confirm the end of the transfer (cs 0xA1)
    let sent = h.sent_payloads();
    assert_eq!(sent.last().unwrap()[0], 0xA1);
}

#[test]
fn read_block_without_size_uses_end_frame_n_bits() {
    // Regression: with no size announced (cs 0xC4, s bit clear), the padding
    // bytes of the last segment used to be copied out and fed to the CRC,
    // producing garbage data and a spurious CRC mismatch.
    let h = Harness::new();
    let mut buf = [0u8; 32];
    let result = run_with_batches(
        &h,
        h.client.read_block(IDX, SUB, &mut buf, true),
        block_upload_batches(0xC4, [0; 4]),
    );
    assert_eq!(result.unwrap(), 10);
    assert_eq!(buf[..10], [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
    // The padding bytes must not leak into the buffer
    assert!(!buf[10..].contains(&0xEE));
}
