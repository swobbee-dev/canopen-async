//! Dispatcher tests: NodeClient::handle_frame classification with the real
//! COB-IDs of the lime (node 1) and enyring (node 0x7F) batteries.

mod common;

use canopen_async::{EmcyMessage, NmtState, NodeClient, NodeEvent};
use common::{MockTx, TestFrame};
use embassy_time::Duration;
use embedded_can::{Frame, StandardId};

fn frame(id: u16, data: &[u8]) -> TestFrame {
    TestFrame::new(StandardId::new(id).unwrap(), data).unwrap()
}

fn client(node_id: u8) -> NodeClient<TestFrame, MockTx> {
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
    let client: NodeClient<TestFrame, MockTx> =
        NodeClient::new(0x11, tx, Duration::from_millis(100)).unwrap();

    futures::executor::block_on(client.send_nmt(NmtCommand::Start)).unwrap();

    let frames = sent.borrow();
    assert_eq!(frames.len(), 1);
    assert_eq!(frames[0].id, StandardId::new(0).unwrap().into());
    assert_eq!(frames[0].data, vec![1, 0x11]);
}
