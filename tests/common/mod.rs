//! Shared test transport: an owned CAN frame and a recording mock TX.
#![allow(dead_code)] // not every integration test uses every helper

use core::cell::RefCell;
use std::rc::Rc;
use std::vec::Vec;

use embedded_can::asynch::CanTx;
use embedded_can::{Frame, Id};

#[derive(Debug, Clone)]
pub struct TestFrame {
    pub id: Id,
    pub data: Vec<u8>,
}

impl Frame for TestFrame {
    fn new(id: impl Into<Id>, data: &[u8]) -> Option<Self> {
        (data.len() <= 8).then(|| TestFrame {
            id: id.into(),
            data: data.to_vec(),
        })
    }

    fn new_remote(id: impl Into<Id>, dlc: usize) -> Option<Self> {
        (dlc <= 8).then(|| TestFrame {
            id: id.into(),
            data: std::vec![0; dlc],
        })
    }

    fn is_extended(&self) -> bool {
        matches!(self.id, Id::Extended(_))
    }

    fn is_remote_frame(&self) -> bool {
        false
    }

    fn id(&self) -> Id {
        self.id
    }

    fn dlc(&self) -> usize {
        self.data.len()
    }

    fn data(&self) -> &[u8] {
        &self.data
    }
}

/// Records every transmitted frame; clone the `sent` handle for assertions.
#[derive(Clone, Default)]
pub struct MockTx {
    pub sent: Rc<RefCell<Vec<TestFrame>>>,
}

impl CanTx for MockTx {
    type Frame = TestFrame;
    type Error = core::convert::Infallible;

    async fn transmit(&mut self, frame: &TestFrame) -> Result<(), Self::Error> {
        self.sent.borrow_mut().push(frame.clone());
        Ok(())
    }
}
