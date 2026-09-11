//! Client-level tests driving `SdoClient` through its public API with a mock
//! CAN transport. Responses are injected via `process_frame`, exactly as an
//! application RX task would.

mod common;

use core::cell::RefCell;
use std::rc::Rc;
use std::vec::Vec;

use canopen_async::{SdoClient, SdoError, SdoOverrides};
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

mod typed {
    use super::*;
    use canopen_async::{Domain, SdoEntry, VisibleString};

    #[test]
    fn read_u8_via_expedited_reply() {
        let h = Harness::new();
        const SOC: SdoEntry<u8> = SdoEntry::new(IDX, SUB, "state of charge");
        // Expedited response, 1 valid byte (cs 0x4F)
        let result = run_with_responses(
            &h,
            h.client.read(SOC),
            vec![Harness::response(0x4F, [87, 0, 0, 0])],
        );
        assert_eq!(result.unwrap(), 87u8);
    }

    #[test]
    fn read_scalar_from_padded_expedited_reply() {
        // The lime BMS pads every expedited response to 4 bytes (cs 0x43,
        // "size indicated: 4") regardless of the object's size. The typed
        // read must decode from the low bytes instead of rejecting the
        // oversized payload.
        let h = Harness::new();
        const SOH: SdoEntry<u8> = SdoEntry::new(IDX, SUB, "state of health");
        let result = run_with_responses(
            &h,
            h.client.read(SOH),
            vec![Harness::response(0x43, [87, 0, 0, 0])],
        );
        assert_eq!(result.unwrap(), 87u8);
    }

    #[test]
    fn read_i16_negative_from_padded_reply() {
        // -80 (deci-degC) as an i16 inside a 4-byte reply: correct for a
        // zero-padding server...
        let h = Harness::new();
        const TEMP: SdoEntry<i16> = SdoEntry::new(IDX, SUB, "temperature");
        let result = run_with_responses(
            &h,
            h.client.read(TEMP),
            vec![Harness::response(0x43, [0xB0, 0xFF, 0x00, 0x00])],
        );
        assert_eq!(result.unwrap(), -80i16);

        // ...and for a sign-extending one.
        let h = Harness::new();
        let result = run_with_responses(
            &h,
            h.client.read(TEMP),
            vec![Harness::response(0x43, [0xB0, 0xFF, 0xFF, 0xFF])],
        );
        assert_eq!(result.unwrap(), -80i16);
    }

    #[test]
    fn read_i32_negative_value() {
        let h = Harness::new();
        const CURRENT: SdoEntry<i32> = SdoEntry::new(IDX, SUB, "current");
        let result = run_with_responses(
            &h,
            h.client.read(CURRENT),
            vec![Harness::response(0x43, (-2500i32).to_le_bytes())],
        );
        assert_eq!(result.unwrap(), -2500);
    }

    #[test]
    fn read_u64_via_segmented_reply() {
        let h = Harness::new();
        const ERROR: SdoEntry<u64> = SdoEntry::new(IDX, SUB, "battery error");
        let value = 0xAABB_CCDD_1122_3344u64.to_le_bytes();
        let result = run_with_responses(
            &h,
            h.client.read(ERROR),
            vec![
                Harness::response(0x41, 8u32.to_le_bytes()),
                // 7 bytes, not last
                Harness::raw_response(&[
                    0x00, value[0], value[1], value[2], value[3], value[4], value[5], value[6],
                ]),
                // 1 byte (n=6), toggle 1, last -> cs = 0001_1101
                Harness::raw_response(&[0x1D, value[7], 0, 0, 0, 0, 0, 0]),
            ],
        );
        assert_eq!(result.unwrap(), 0xAABB_CCDD_1122_3344);
    }

    #[test]
    fn write_scalars_pick_transfer_mode() {
        let h = Harness::new();
        const TTYPE: SdoEntry<u8> = SdoEntry::new(IDX, SUB, "small");
        let result = run_with_responses(
            &h,
            h.client.write(TTYPE, 254u8),
            vec![Harness::response(0x60, [0; 4])],
        );
        assert!(result.is_ok());
        // 1-byte expedited download: cs 0x2F
        assert_eq!(h.sent_payloads()[0][..5], [0x2F, 0x00, 0x20, SUB, 254]);

        // u64 goes segmented: initiate (0x21, size 8) + two segments
        let h = Harness::new();
        const VALIDITY: SdoEntry<u64> = SdoEntry::new(IDX, SUB, "validity");
        let result = run_with_responses(
            &h,
            h.client.write(VALIDITY, 0x1122_3344_5566_7788),
            vec![
                Harness::response(0x60, [0; 4]),
                Harness::raw_response(&[0x20, 0, 0, 0, 0, 0, 0, 0]), // seg ack t0
                Harness::raw_response(&[0x30, 0, 0, 0, 0, 0, 0, 0]), // seg ack t1
            ],
        );
        assert!(result.is_ok());
        let sent = h.sent_payloads();
        assert_eq!(sent[0][0], 0x21); // initiate segmented download, size indicated
        assert_eq!(sent[0][4..8], 8u32.to_le_bytes());
    }

    #[test]
    fn read_string_and_domain_delegate_to_transfers() {
        let h = Harness::new();
        const UID: SdoEntry<VisibleString> = SdoEntry::new(IDX, SUB, "battery UID");
        let mut buf = [0u8; 16];
        let result = run_with_responses(
            &h,
            h.client.read_bytes(UID, &mut buf),
            vec![
                Harness::response(0x41, 4u32.to_le_bytes()),
                // 4 bytes (n=3), last, toggle 0 -> cs 0000_0111
                Harness::raw_response(&[0x07, b'A', b'B', b'C', b'D', 0, 0, 0]),
            ],
        );
        assert_eq!(result.unwrap(), 4);
        assert_eq!(&buf[..4], b"ABCD");

        // Domain read goes through block transfer
        let h = Harness::new();
        const NONCE: SdoEntry<Domain> = SdoEntry::new(IDX, SUB, "nonce");
        let mut buf = [0u8; 32];
        let result = run_with_batches(
            &h,
            h.client.read_domain(NONCE, &mut buf, true),
            block_upload_batches(0xC6, 10u32.to_le_bytes()),
        );
        assert_eq!(result.unwrap(), 10);
        assert_eq!(buf[..10], [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
    }
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

/// Batches for a complete block upload of a larger `payload`,
/// cut into sub-blocks of `blksize` segments.
fn block_upload_batches_of(payload: &[u8], blksize: usize) -> Vec<Vec<TestFrame>> {
    let crc = block_crc(payload).to_le_bytes();
    let mut batches = vec![vec![Harness::response(0xC6, (payload.len() as u32).to_le_bytes())]];

    let chunks: Vec<&[u8]> = payload.chunks(7).collect();
    let last = chunks.len() - 1;
    let segments: Vec<TestFrame> = chunks
        .iter()
        .enumerate()
        .map(|(i, chunk)| {
            let seqno = (i % blksize) as u8 + 1;
            // Padding in the final segment needs to be dropped
            let mut frame = [0xEEu8; 8];
            frame[0] = if i == last { 0x80 | seqno } else { seqno };
            frame[1..1 + chunk.len()].copy_from_slice(chunk);
            Harness::raw_response(&frame)
        })
        .collect();
    batches.extend(segments.chunks(blksize).map(<[TestFrame]>::to_vec));

    // End frame: cs = 110_nnn_01, nnn = unused bytes in the last segment.
    let unused = ((7 - payload.len() % 7) % 7) as u8;
    batches.push(vec![Harness::raw_response(&[0xC1 | (unused << 2), crc[0], crc[1], 0, 0, 0, 0, 0])]);
    batches
}

#[test]
fn read_block_honours_a_client_blksize_override() {
    // Propose a sub-block no larger than the segment channel size keeps
    // bursts within what the channel.
    let payload: Vec::<u8> = (1..=60u8).collect();
    let mut h = Harness::new();
    h.client.use_overrides(SdoOverrides {
        read_blksize: Some(4),
        write_segment_delay: None,
    });
    let mut buf = [0u8; 1000];
    let result = run_with_batches(&h, h.client.read_block(IDX, SUB, &mut buf, true), block_upload_batches_of(&payload, 4));

    assert_eq!(result.unwrap(), payload.len());
    assert_eq!(&buf[..payload.len()], &payload[..]);

    // Ensure blksize has reached the server.
    assert_eq!(h.sent_payloads()[0][4], 4);
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
