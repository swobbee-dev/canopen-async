#![no_std]

use core::cell::RefCell;
use core::marker::PhantomData;
use embassy_time::Duration;
use embedded_can::{Frame, Id, StandardId, asynch::CanTx};

pub mod sync;

pub use sync::{AsyncMutex, AsyncSignal};

#[allow(dead_code)]
#[derive(Debug, PartialEq, Clone)]
pub enum SdoError<E: Clone> {
    Timeout,
    RequestPending,
    TxError(E),
    InvalidResponse,
    SdoAbort(u32),
    BufferSizeWrong,
    InvalidNodeId,
}

#[cfg(feature = "defmt")]
impl<E> defmt::Format for SdoError<E> {
    fn format(&self, f: defmt::Formatter) {
        match self {
            SdoError::Timeout => defmt::write!(f, "Timeout"),
            SdoError::RequestPending => defmt::write!(f, "RequestPending"),
            SdoError::InvalidResponse => defmt::write!(f, "InvalidResponse"),
            SdoError::SdoAbort(code) => defmt::write!(f, "SdoAbort(0x{:08X})", code),
            SdoError::TxError(_) => defmt::write!(f, "TxError"),
            SdoError::BufferSizeWrong => defmt::write!(f, "BufferSizeWrong"),
            SdoError::InvalidNodeId => defmt::write!(f, "InvalidNodeId"),
        }
    }
}

pub struct SdoClient<FRAME, TX, M, S> 
where
    TX: CanTx<Frame = FRAME>,
    TX::Error: Clone,
    M: AsyncMutex,
    S: AsyncSignal,
{
    node_id: u8,
    request_lock: M::Mutex<()>,
    state: RequestState<S, TX::Error>,
    can_tx: M::Mutex<TX>,
    timeout: Duration,
    _phantom: PhantomData<(FRAME, TX)>,
}

struct RequestState<S, TxError: Clone> 
where
    S: AsyncSignal,
{
    pending: RefCell<Option<Pending>>,
    word_signal: S::Signal<Result<u32, SdoError<TxError>>>,
    segment_signal: S::Signal<Result<Segment, SdoError<TxError>>>,
    ack_signal: S::Signal<Result<(), SdoError<TxError>>>,
}

enum SdoRequest<'a> {
    UploadExpedited {
        index: u16,
        sub: u8,
    },
    DownloadExpedited {
        index: u16,
        sub: u8,
        data: &'a [u8],
    },
    InitiateSegmentedDownload {
        index: u16,
        sub: u8,
        size: u32,
    },
    UploadSegment {
        toggle: bool,
    },
    DownloadSegment {
        toggle: bool,
        last: bool,
        data: &'a [u8],
    },
}

#[derive(Debug, Clone, Copy)]
enum Pending {
    ExpeditedRead { index: u16, sub: u8 },
    ExpeditedWrite { index: u16, sub: u8 },
    SegmentedDownloadInit { index: u16, sub: u8 },
    UploadSegment { toggle: bool },
    DownloadSegment { toggle: bool },
    // FUTURE: A state for block transfers would be added here.
    // e.g. BlockTransfer { state: BlockState }
}

#[derive(Copy, Clone)]
pub struct Segment {
    last: bool,
    len: usize,
    data: [u8; 7],
}

struct PendingGuard<'a> {
    pending: &'a RefCell<Option<Pending>>,
}
impl<'a> PendingGuard<'a> {
    fn new(pending: &'a RefCell<Option<Pending>>) -> Self {
        Self { pending }
    }
}
impl<'a> Drop for PendingGuard<'a> {
    fn drop(&mut self) {
        *self.pending.borrow_mut() = None;
    }
}

impl<FRAME: Frame, TX, M, S> SdoClient<FRAME, TX, M, S>
where
    TX: CanTx<Frame = FRAME>,
    TX::Error: Clone,
    M: AsyncMutex,
    S: AsyncSignal,
{
    pub fn new(
        node_id: u8, 
        timeout: Duration,
        can_tx_mutex: M::Mutex<TX>,
    ) -> Result<Self, SdoError<TX::Error>> {
        if node_id > 127 {
            return Err(SdoError::InvalidNodeId);
        }

        Ok(Self {
            node_id,
            request_lock: M::new(()),
            state: RequestState {
                pending: RefCell::new(None),
                word_signal: S::new(),
                segment_signal: S::new(),
                ack_signal: S::new(),
            },
            can_tx: can_tx_mutex,
            timeout,
            _phantom: PhantomData,
        })
    }

    async fn request_response<'a, R: Clone>(
        &self,
        request_to_send: SdoRequest<'a>,
        pending_state: Pending,
        response_signal: &S::Signal<Result<R, SdoError<TX::Error>>>,
    ) -> Result<R, SdoError<TX::Error>> {
        *self.state.pending.borrow_mut() = Some(pending_state);
        let _guard = PendingGuard::new(&self.state.pending);
        S::reset(response_signal);

        // Transmit the request frame
        match request_to_send {
            SdoRequest::UploadExpedited { index, sub } => {
                self.send_sdo_upload_request(index, sub).await
            }
            SdoRequest::DownloadExpedited { index, sub, data } => {
                self.send_sdo_download_request(index, sub, data).await
            }
            SdoRequest::InitiateSegmentedDownload { index, sub, size } => {
                self.send_initiate_segmented_download(index, sub, size)
                    .await
            }
            SdoRequest::UploadSegment { toggle } => {
                self.send_sdo_request_upload_segment(toggle).await
            }
            SdoRequest::DownloadSegment { toggle, last, data } => {
                self.send_sdo_download_segment(data, toggle, last).await
            }
        }
        .map_err(SdoError::TxError)?;

        // Wait for the response
        match embassy_time::with_timeout(self.timeout, S::wait(response_signal)).await {
            Ok(inner_result) => inner_result,
            Err(_) => Err(SdoError::Timeout),
        }
    }

    pub async fn read_expedited(&self, index: u16, sub: u8) -> Result<u32, SdoError<TX::Error>> {
        let _guard = M::lock(&self.request_lock).await;
        self.read_expedited_locked(index, sub).await
    }

    pub async fn write_expedited(
        &self,
        index: u16,
        sub: u8,
        data: &[u8],
    ) -> Result<(), SdoError<TX::Error>> {
        let _guard = M::lock(&self.request_lock).await;
        self.write_expedited_locked(index, sub, data).await
    }

    pub async fn read_segmented(
        &self,
        index: u16,
        sub: u8,
        buf: &mut [u8],
    ) -> Result<(), SdoError<TX::Error>> {
        let _guard = M::lock(&self.request_lock).await;
        self.read_segmented_locked(index, sub, buf).await
    }

    #[allow(dead_code)]
    pub async fn write_segmented(
        &self,
        index: u16,
        sub: u8,
        data: &[u8],
    ) -> Result<(), SdoError<TX::Error>> {
        let _guard = M::lock(&self.request_lock).await;
        self.write_segmented_locked(index, sub, data).await
    }

    #[allow(dead_code)]
    pub async fn read_block(
        &self,
        index: u16,
        sub: u8,
        buf: &mut [u8],
    ) -> Result<(), SdoError<TX::Error>> {
        let _guard = M::lock(&self.request_lock).await;
        self.read_block_locked(index, sub, buf).await
    }

    #[allow(dead_code)]
    pub async fn write_block(
        &self,
        index: u16,
        sub: u8,
        data: &[u8],
    ) -> Result<(), SdoError<TX::Error>> {
        let _guard = M::lock(&self.request_lock).await;
        self.write_block_locked(index, sub, data).await
    }

    pub async fn on_frame_received(&self, frame: FRAME) {
        let expected_id = 0x580 + self.node_id as u16;
        let Id::Standard(id) = frame.id() else {
            return; // Not a standard frame
        };
        if id.as_raw() != expected_id {
            return; // Not an SDO response for this node
        }

        let Some(pending_request) = self.state.pending.borrow_mut().take() else {
            return;
        };

        if frame.data().len() != 8 {
            let err = SdoError::InvalidResponse;
            self.signal_error(pending_request, err);
            return;
        }

        let command = frame.data()[0];
        if command == 0x80 {
            // SDO Abort
            let abort_code = u32::from_le_bytes(frame.data()[4..8].try_into().unwrap());
            self.signal_error(pending_request, SdoError::SdoAbort(abort_code));
            return;
        }

        match pending_request {
            Pending::ExpeditedRead { index, sub } => {
                let response_index = u16::from_le_bytes(frame.data()[1..3].try_into().unwrap());
                let response_sub = frame.data()[3];
                if response_index != index || response_sub != sub {
                    S::signal(&self.state.word_signal, Err(SdoError::InvalidResponse));
                    return;
                }

                match command {
                    // Expedited Upload Response (e.g. 0x43, 0x47, 0x4B, 0x4F)
                    0x43 | 0x47 | 0x4B | 0x4F => {
                        let n_unused = ((command & 0x0C) >> 2) as usize;
                        let data_len = 4 - n_unused;
                        let mut bytes = [0u8; 4];
                        bytes[..data_len].copy_from_slice(&frame.data()[4..4 + data_len]);
                        S::signal(&self.state.word_signal, Ok(u32::from_le_bytes(bytes)));
                    }
                    // Segmented Upload Initiation Response
                    0x41 => {
                        let size = u32::from_le_bytes(frame.data()[4..8].try_into().unwrap());
                        S::signal(&self.state.word_signal, Ok(size));
                    }
                    // Any other command is invalid for this state.
                    _ => {
                        S::signal(&self.state.word_signal, Err(SdoError::InvalidResponse));
                    }
                }
            }

            Pending::ExpeditedWrite { index, sub }
            | Pending::SegmentedDownloadInit { index, sub } => {
                let response_index = u16::from_le_bytes(frame.data()[1..3].try_into().unwrap());
                let response_sub = frame.data()[3];
                if response_index != index || response_sub != sub {
                    S::signal(&self.state.ack_signal, Err(SdoError::InvalidResponse));
                    return;
                }

                // The only valid response is a download confirmation.
                if command == 0x60 {
                    S::signal(&self.state.ack_signal, Ok(()));
                } else {
                    S::signal(&self.state.ack_signal, Err(SdoError::InvalidResponse));
                }
            }

            Pending::UploadSegment { toggle } => {
                // Segment Upload Response: cs = 000t nnnc b
                if (command & 0xE0) == 0x00 {
                    let response_toggle = (command & 0x10) != 0;
                    if response_toggle != toggle {
                        S::signal(&self.state.segment_signal, Err(SdoError::InvalidResponse));
                        return;
                    }

                    let last = (command & 0x01) != 0;
                    let n_unused = ((command & 0x0E) >> 1) as usize;
                    let len = 7 - n_unused;
                    let mut data = [0u8; 7];
                    data[..len].copy_from_slice(&frame.data()[1..1 + len]);
                    let segment = Segment { last, len, data };
                    S::signal(&self.state.segment_signal, Ok(segment));
                } else {
                    S::signal(&self.state.segment_signal, Err(SdoError::InvalidResponse));
                }
            }

            Pending::DownloadSegment { toggle } => {
                // Download Segment Response: cs = 001t 0000 b
                if (command & 0b1110_1111) == 0b0010_0000 {
                    let response_toggle = (command & 0b0001_0000) != 0;
                    if response_toggle == toggle {
                        S::signal(&self.state.ack_signal, Ok(()));
                    } else {
                        S::signal(&self.state.ack_signal, Err(SdoError::InvalidResponse));
                    }
                } else {
                    S::signal(&self.state.ack_signal, Err(SdoError::InvalidResponse));
                }
            }
        };
    }

    async fn read_expedited_locked(&self, index: u16, sub: u8) -> Result<u32, SdoError<TX::Error>> {
        self.request_response(
            SdoRequest::UploadExpedited { index, sub },
            Pending::ExpeditedRead { index, sub },
            &self.state.word_signal,
        )
        .await
    }

    async fn write_expedited_locked(
        &self,
        index: u16,
        sub: u8,
        data: &[u8],
    ) -> Result<(), SdoError<TX::Error>> {
        if data.is_empty() || data.len() > 4 {
            return Err(SdoError::BufferSizeWrong);
        }

        self.request_response(
            SdoRequest::DownloadExpedited { index, sub, data },
            Pending::ExpeditedWrite { index, sub },
            &self.state.ack_signal,
        )
        .await
    }

    async fn read_segmented_locked(
        &self,
        index: u16,
        sub: u8,
        buf: &mut [u8],
    ) -> Result<(), SdoError<TX::Error>> {
        // Initiate the SDO with an expedited request to get the size of the data
        let len = self.read_expedited_locked(index, sub).await?;
        if len as usize > buf.len() {
            return Err(SdoError::BufferSizeWrong);
        }
        let buf = &mut buf[..len as usize];

        // Initiation successful, start requesting segments
        let mut offset = 0usize;
        let mut toggle = false;

        loop {
            let segment = self
                .request_response(
                    SdoRequest::UploadSegment { toggle },
                    Pending::UploadSegment { toggle },
                    &self.state.segment_signal,
                )
                .await?;

            buf[offset..offset + segment.len].copy_from_slice(&segment.data[..segment.len]);
            offset += segment.len;

            if segment.last {
                break;
            }

            toggle = !toggle;
        }

        Ok(())
    }

    async fn write_segmented_locked(
        &self,
        index: u16,
        sub: u8,
        data: &[u8],
    ) -> Result<(), SdoError<TX::Error>> {
        // Initiate download
        self.request_response(
            SdoRequest::InitiateSegmentedDownload {
                index,
                sub,
                size: data.len() as u32,
            },
            Pending::SegmentedDownloadInit { index, sub },
            &self.state.ack_signal,
        )
        .await?;

        // Send segments
        let mut toggle = false;
        let mut chunks = data.chunks(7).peekable();
        while let Some(chunk) = chunks.next() {
            let last = chunks.peek().is_none();
            self.request_response(
                SdoRequest::DownloadSegment {
                    toggle,
                    last,
                    data: chunk,
                },
                Pending::DownloadSegment { toggle },
                &self.state.ack_signal,
            )
            .await?;

            toggle = !toggle;
        }
        Ok(())
    }

    async fn read_block_locked(
        &self,
        _index: u16,
        _sub: u8,
        _buf: &mut [u8],
    ) -> Result<(), SdoError<TX::Error>> {
        unimplemented!("SDO Block Transfer is not yet implemented");
    }

    async fn write_block_locked(
        &self,
        _index: u16,
        _sub: u8,
        _data: &[u8],
    ) -> Result<(), SdoError<TX::Error>> {
        unimplemented!("SDO Block Transfer is not yet implemented");
    }

    fn signal_error(&self, pending: Pending, err: SdoError<TX::Error>) {
        match pending {
            Pending::ExpeditedRead { .. } => S::signal(&self.state.word_signal, Err(err)),
            Pending::ExpeditedWrite { .. } | Pending::SegmentedDownloadInit { .. } => {
                S::signal(&self.state.ack_signal, Err(err))
            }
            Pending::UploadSegment { .. } => S::signal(&self.state.segment_signal, Err(err)),
            Pending::DownloadSegment { .. } => S::signal(&self.state.ack_signal, Err(err)),
        }
    }

    // ## --- HELPER SENDER FUNCTIONS --- ##

    async fn send_sdo_upload_request(&self, index: u16, subindex: u8) -> Result<(), TX::Error> {
        let id = 0x600 + (self.node_id as u16);
        let mut payload = [0u8; 8];
        payload[0] = 0x40; // Initiate Upload Request
        payload[1..3].copy_from_slice(&index.to_le_bytes());
        payload[3] = subindex;
        let frame = FRAME::new(Id::Standard(StandardId::new(id).unwrap()), &payload).unwrap();
        M::lock(&self.can_tx).await.transmit(&frame).await
    }

    async fn send_sdo_download_request(
        &self,
        index: u16,
        subindex: u8,
        data: &[u8],
    ) -> Result<(), TX::Error> {
        let id = 0x600 + (self.node_id as u16);
        let mut payload = [0u8; 8];

        let len = data.len();
        let n = 4 - len;
        // Expedited download CS: 001_0_nn_11 b
        payload[0] = 0b0010_0011 | ((n as u8) << 2);

        payload[1..3].copy_from_slice(&index.to_le_bytes());
        payload[3] = subindex;
        payload[4..4 + len].copy_from_slice(data);

        let frame = FRAME::new(Id::Standard(StandardId::new(id).unwrap()), &payload).unwrap();
        M::lock(&self.can_tx).await.transmit(&frame).await
    }

    async fn send_initiate_segmented_download(
        &self,
        index: u16,
        sub: u8,
        size: u32,
    ) -> Result<(), TX::Error> {
        let id = 0x600 + (self.node_id as u16);
        let mut payload = [0u8; 8];
        payload[0] = 0x21; // Initiate Segmented Download, size is indicated
        payload[1..3].copy_from_slice(&index.to_le_bytes());
        payload[3] = sub;
        payload[4..8].copy_from_slice(&size.to_le_bytes());

        let frame = FRAME::new(Id::Standard(StandardId::new(id).unwrap()), &payload).unwrap();
        M::lock(&self.can_tx).await.transmit(&frame).await
    }

    async fn send_sdo_download_segment(
        &self,
        segment_data: &[u8],
        toggle: bool,
        last: bool,
    ) -> Result<(), TX::Error> {
        let id = 0x600 + (self.node_id as u16);
        let mut payload = [0u8; 8];

        let n = 7 - segment_data.len();
        // Download Segment CS: 000t nnnc b
        let mut cs: u8 = 0;
        if toggle {
            cs |= 0b0001_0000;
        }
        cs |= (n as u8) << 1;
        if last {
            cs |= 0b0000_0001;
        }

        payload[0] = cs;
        payload[1..1 + segment_data.len()].copy_from_slice(segment_data);

        let frame = FRAME::new(Id::Standard(StandardId::new(id).unwrap()), &payload).unwrap();
        M::lock(&self.can_tx).await.transmit(&frame).await
    }

    async fn send_sdo_request_upload_segment(&self, toggle: bool) -> Result<(), TX::Error> {
        let id = 0x600 + (self.node_id as u16);
        let mut payload = [0u8; 8];
        // Upload SDO Segment Request: 011t 0000 b
        payload[0] = 0x60 | if toggle { 0x10 } else { 0x00 };
        let frame = FRAME::new(Id::Standard(StandardId::new(id).unwrap()), &payload).unwrap();
        M::lock(&self.can_tx).await.transmit(&frame).await
    }
}