//! Dispatcher tests: NodeClient::handle_frame classification with the real
//! COB-IDs of the lime (node 1) and enyring (node 0x7F) batteries.

mod common;

use canopen_async::{EmcyMessage, NmtState, NodeClient, NodeEvent, PdoConfig, PdoMappingEntry};
use common::{MockTx, TestFrame};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_time::Duration;
use embedded_can::{Frame, StandardId};
use futures::FutureExt;
use futures::executor::block_on;

type Client = NodeClient<CriticalSectionRawMutex, TestFrame, MockTx>;

fn frame(id: u16, data: &[u8]) -> TestFrame {
    TestFrame::new(StandardId::new(id).unwrap(), data).unwrap()
}

fn client(node_id: u8) -> Client {
    NodeClient::new(node_id, MockTx::default(), Duration::from_millis(100)).unwrap()
}

#[test]
fn classifies_lime_battery_frames() {
    let client = client(1);

    // Heartbeat 0x701 (the DBC called this "GUARD"), DLC 1
    assert_eq!(
        client.handle_frame(&frame(0x701, &[5])),
        Some(NodeEvent::Heartbeat(NmtState::Operational))
    );

    // Boot-up message
    assert_eq!(
        client.handle_frame(&frame(0x701, &[0])),
        Some(NodeEvent::Heartbeat(NmtState::Bootup))
    );

    // TPDO1..4 hand out the raw payload for app-side decoding
    let pdo1 = frame(0x181, &[1, 2, 3, 4, 5, 6, 7, 8]);
    assert_eq!(
        client.handle_frame(&pdo1),
        Some(NodeEvent::Tpdo { num: 1, data: &[1, 2, 3, 4, 5, 6, 7, 8] })
    );
    for (id, num) in [(0x281u16, 2u8), (0x381, 3), (0x481, 4)] {
        let f = frame(id, &[0; 8]);
        assert!(matches!(
            client.handle_frame(&f),
            Some(NodeEvent::Tpdo { num: n, .. }) if n == num
        ));
    }

    // Lime's nonstandard 3-byte EMCY frame
    assert_eq!(
        client.handle_frame(&frame(0x081, &[0xFF, 0x10, 0x20])),
        Some(NodeEvent::Emcy(EmcyMessage {
            error_code: 0x10FF,
            error_register: 0x20,
            vendor: [0; 5],
        }))
    );

    assert_eq!(client.handle_frame(&frame(0x080, &[])), Some(NodeEvent::Sync));

    // Frames of other nodes are not classified into this node's set
    assert_eq!(client.handle_frame(&frame(0x182, &[0; 8])), Some(NodeEvent::Unhandled));
    assert_eq!(client.handle_frame(&frame(0x702, &[5])), Some(NodeEvent::Unhandled));
}

#[test]
fn classifies_enyring_heartbeat() {
    // Enyring uses node id 0x7F; its heartbeat COB-ID 0x77F is the standard
    // 0x700 + node id, previously special-cased by hand in the driver.
    let client = client(0x7F);
    assert_eq!(
        client.handle_frame(&frame(0x77F, &[127])),
        Some(NodeEvent::Heartbeat(NmtState::PreOperational))
    );
}

#[test]
fn sdo_responses_are_consumed_internally() {
    let client = client(1);
    // An (unsolicited) SDO response: consumed by the SDO client, no event
    let sdo_response = frame(0x581, &[0x43, 0x00, 0x20, 0x01, 1, 2, 3, 4]);
    assert_eq!(client.handle_frame(&sdo_response), None);
}

#[test]
fn nmt_command_is_addressed_to_the_node() {
    use canopen_async::NmtCommand;

    let tx = MockTx::default();
    let sent = tx.sent.clone();
    let client: Client = NodeClient::new(0x11, tx, Duration::from_millis(100)).unwrap();

    block_on(client.send_nmt(NmtCommand::Start)).unwrap();

    let frames = sent.borrow();
    assert_eq!(frames.len(), 1);
    assert_eq!(frames[0].id, StandardId::new(0).unwrap().into());
    assert_eq!(frames[0].data, vec![1, 0x11]);
}

#[test]
fn monitors_are_fed_by_the_dispatcher() {
    let client = client(1);

    // Heartbeat updates the heartbeat monitor
    assert_eq!(client.heartbeat.last_state(), None);
    client.handle_frame(&frame(0x701, &[5]));
    assert_eq!(client.heartbeat.last_state(), Some(NmtState::Operational));
    assert!(client.heartbeat.is_alive(Duration::from_secs(1)));

    // EMCY updates the emcy monitor
    assert!(!client.emcy.error_active());
    client.handle_frame(&frame(0x081, &[0x01, 0x10, 0x81]));
    assert!(client.emcy.error_active());
    assert_eq!(client.emcy.latest().unwrap().error_code, 0x1001);

    // An error-reset EMCY clears the active flag
    client.handle_frame(&frame(0x081, &[0, 0, 0]));
    assert!(!client.emcy.error_active());
}

#[test]
fn sync_and_rpdo_transmission() {
    let tx = MockTx::default();
    let sent = tx.sent.clone();
    let client: Client = NodeClient::new(0x11, tx, Duration::from_millis(100)).unwrap();

    block_on(client.send_sync()).unwrap();
    block_on(client.send_rpdo(1, &[0xAA, 0xBB])).unwrap();
    block_on(client.send_rpdo(4, &[1, 2, 3, 4, 5, 6, 7, 8])).unwrap();
    assert!(block_on(client.send_rpdo(5, &[0])).is_err());

    let frames = sent.borrow();
    assert_eq!(frames.len(), 3);
    assert_eq!(frames[0].id, StandardId::new(0x080).unwrap().into());
    assert!(frames[0].data.is_empty());
    assert_eq!(frames[1].id, StandardId::new(0x211).unwrap().into());
    assert_eq!(frames[1].data, vec![0xAA, 0xBB]);
    assert_eq!(frames[2].id, StandardId::new(0x511).unwrap().into());
}

/// Drive a NodeClient future while acking every SDO download request the
/// client sends (echoing the request's multiplexer), like a compliant server.
fn run_with_autoack<F: core::future::Future>(client: &Client, sent: &common::SentFrames, fut: F) -> F::Output {
    let feeder = async {
        let mut acked = 0;
        loop {
            let request = {
                let frames = sent.borrow();
                frames.get(acked).cloned()
            };
            match request {
                Some(req) if (req.data[0] & 0xE0) == 0x20 => {
                    acked += 1;
                    let ack = frame(0x591, &[0x60, req.data[1], req.data[2], req.data[3], 0, 0, 0, 0]);
                    client.handle_frame(&ack);
                }
                Some(_) => acked += 1, // not a download request; skip
                None => yield_once().await,
            }
        }
    };
    block_on(async {
        futures::select_biased! {
            result = fut.fuse() => result,
            _ = feeder.fuse() => unreachable!("feeder never completes"),
        }
    })
}

async fn yield_once() {
    let mut yielded = false;
    core::future::poll_fn(|cx| {
        if yielded {
            core::task::Poll::Ready(())
        } else {
            yielded = true;
            cx.waker().wake_by_ref();
            core::task::Poll::Pending
        }
    })
    .await
}

#[test]
fn configure_tpdo_writes_the_cia301_sequence() {
    let tx = MockTx::default();
    let sent = tx.sent.clone();
    // Node 0x11, TPDO2: comm object 0x1801, mapping object 0x1A01,
    // default COB-ID 0x280 + 0x11 = 0x291
    let client: Client = NodeClient::new(0x11, tx, Duration::from_millis(100)).unwrap();

    let config = PdoConfig {
        cob_id: None,
        transmission_type: 254,
        inhibit_time: None,
        event_timer: Some(100),
        mappings: &[
            PdoMappingEntry::new(0x2000, 1, 16),
            PdoMappingEntry::new(0x2000, 2, 32),
        ],
    };
    run_with_autoack(&client, &sent, client.configure_tpdo(2, &config)).unwrap();

    let payloads: Vec<Vec<u8>> = sent.borrow().iter().map(|f| f.data.clone()).collect();
    let expected: Vec<Vec<u8>> = vec![
        // Invalidate COB-ID: 0x291 | bit31, 4-byte expedited write to 0x1801:1
        vec![0x23, 0x01, 0x18, 1, 0x91, 0x02, 0x00, 0x80],
        // Transmission type 254, 1 byte to 0x1801:2
        vec![0x2F, 0x01, 0x18, 2, 254, 0, 0, 0],
        // Event timer 100 ms, 2 bytes to 0x1801:5
        vec![0x2B, 0x01, 0x18, 5, 100, 0, 0, 0],
        // Clear mapping count on 0x1A01:0
        vec![0x2F, 0x01, 0x1A, 0, 0, 0, 0, 0],
        // Mapping 1: 0x2000:1, 16 bits -> 0x20000110
        vec![0x23, 0x01, 0x1A, 1, 0x10, 0x01, 0x00, 0x20],
        // Mapping 2: 0x2000:2, 32 bits -> 0x20000220
        vec![0x23, 0x01, 0x1A, 2, 0x20, 0x02, 0x00, 0x20],
        // Mapping count = 2
        vec![0x2F, 0x01, 0x1A, 0, 2, 0, 0, 0],
        // Revalidate COB-ID 0x291
        vec![0x23, 0x01, 0x18, 1, 0x91, 0x02, 0x00, 0x00],
    ];
    assert_eq!(payloads, expected);
}
