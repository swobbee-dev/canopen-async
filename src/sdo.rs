//! Async SDO client (CiA 301, 7.2.4).

use core::cell::RefCell;
use crc::{Algorithm, Crc};
use embassy_sync::{
    blocking_mutex::raw::NoopRawMutex, channel::Channel, mutex::Mutex, signal::Signal,
};
use embassy_time::Duration;
use embedded_can::{Frame, Id, StandardId, asynch::CanTx};
use crate::protocol::{
    ABORT_CRC_ERROR, ABORT_GENERAL_ERROR, ABORT_INVALID_BLOCK_SIZE, ABORT_INVALID_CS,
    ABORT_OUT_OF_MEMORY, ABORT_SEQ_NUM_ERROR, ABORT_TIMEOUT, BlockAck, BlockInit, BlockUploadInit,
    BlockUploadSegment, ParseOutcome, Pending, Segment, encode_abort,
    encode_block_download_segment, encode_block_upload_ack, encode_download_expedited,
    encode_download_segment, encode_end_block_download, encode_end_block_upload_confirmation,
    encode_initiate_block_download, encode_initiate_block_upload,
    encode_initiate_segmented_download, encode_start_block_upload, encode_upload_request,
    encode_upload_segment_request, parse_response, UploadInit,
};

// CiA 301 Version 4.2.0, page 72ff
// cs: command specifier
const NMT_CS_START: u8 = 1;
const NMT_CS_STOP: u8 = 2;
const NMT_CS_ENTER_PRE_OPERATIONAL: u8 = 128;
const NMT_CS_RESET_NODE: u8 = 129;
const NMT_CS_RESET_COMMUNICATION: u8 = 130;

#[allow(dead_code)]
#[derive(Debug, PartialEq)]
pub enum SdoError<E> {
    Timeout,
    RequestPending,
    TxError(E),
    InvalidResponse,
    SdoAbort(u32),
    BufferSizeWrong,
    InvalidNodeId,
    StreamError,
    /// The server answered an expedited read with a segmented transfer,
    /// i.e. the object is larger than 4 bytes. Use [`SdoClient::read_segmented`].
    NotExpedited,
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
            SdoError::StreamError => defmt::write!(f, "StreamError"),
            SdoError::NotExpedited => defmt::write!(f, "NotExpedited"),
        }
    }
}

/// Position from which to seek.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SeekFrom {
    /// Seek from the beginning of the stream.
    Start(u64),
    /// Seek from the end of the stream.
    End(i64),
    /// Seek from the current position.
    Current(i64),
}

/// A trait for reading data in chunks for SDO block downloads.
pub trait StreamReader<E> {
    /// Reads the next chunk of data into the buffer.
    ///
    /// Returns the number of bytes read. A return value of 0 indicates the end of the stream.
    async fn read(&mut self, buf: &mut [u8]) -> Result<usize, SdoError<E>>;
}

/// A trait for seeking within a stream.
pub trait StreamSeeker<E> {
    /// Seek to a new position in the stream.
    ///
    /// Returns the new position from the start of the stream.
    async fn seek(&mut self, pos: SeekFrom) -> Result<u64, SdoError<E>>;

    /// Returns the current position of the stream.
    /// This is equivalent to `seek(SeekFrom::Current(0))`.
    async fn stream_position(&mut self) -> Result<u64, SdoError<E>> {
        self.seek(SeekFrom::Current(0)).await
    }
}

pub struct SdoClient<FRAME, TX: CanTx<Frame = FRAME>> {
    node_id: u8,
    request_lock: Mutex<NoopRawMutex, ()>,
    state: RequestState<FRAME, TX>,
    can_tx: Mutex<NoopRawMutex, TX>,
    timeout: Duration,
}

// A queue size of 4 should be sufficient for most CAN bus conditions.
const BLOCK_SEGMENT_QUEUE_SIZE: usize = 4;

struct RequestState<FRAME, TX: CanTx<Frame = FRAME>> {
    pending: RefCell<Option<Pending>>,
    sig_upload_init: Signal<NoopRawMutex, Result<UploadInit, SdoError<TX::Error>>>,
    sig_seg: Signal<NoopRawMutex, Result<Segment, SdoError<TX::Error>>>,
    sig_ack: Signal<NoopRawMutex, Result<(), SdoError<TX::Error>>>,
    sig_block_init: Signal<NoopRawMutex, Result<BlockInit, SdoError<TX::Error>>>,
    sig_block_ack: Signal<NoopRawMutex, Result<BlockAck, SdoError<TX::Error>>>,
    sig_block_upload_init: Signal<NoopRawMutex, Result<BlockUploadInit, SdoError<TX::Error>>>,
    block_upload_seg_chan: Channel<
        NoopRawMutex,
        Result<BlockUploadSegment, SdoError<TX::Error>>,
        BLOCK_SEGMENT_QUEUE_SIZE,
    >,
    /// End-of-block-upload: (crc, unused bytes in last segment)
    sig_block_upload_end: Signal<NoopRawMutex, BlockUploadEndResult<TX::Error>>,
}

/// End-of-block-upload outcome: (crc, unused bytes in last segment).
type BlockUploadEndResult<E> = Result<(u16, u8), SdoError<E>>;

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

impl<FRAME: Frame, TX: CanTx<Frame = FRAME>> SdoClient<FRAME, TX> {
    pub fn new(node_id: u8, tx: TX, timeout: Duration) -> Result<Self, SdoError<TX::Error>> {
        if node_id > 127 {
            return Err(SdoError::InvalidNodeId);
        }

        Ok(Self {
            node_id,
            request_lock: Mutex::new(()),
            state: RequestState {
                pending: RefCell::new(None),
                sig_upload_init: Signal::new(),
                sig_seg: Signal::new(),
                sig_ack: Signal::new(),
                sig_block_init: Signal::new(),
                sig_block_ack: Signal::new(),
                sig_block_upload_init: Signal::new(),
                block_upload_seg_chan: Channel::new(),
                sig_block_upload_end: Signal::new(),
            },
            can_tx: Mutex::new(tx),
            timeout,
        })
    }

    async fn request_response<'a, R>(
        &self,
        request_to_send: SdoRequest<'a>,
        pending_state: Pending,
        response_signal: &Signal<NoopRawMutex, Result<R, SdoError<TX::Error>>>,
    ) -> Result<R, SdoError<TX::Error>> {
        *self.state.pending.borrow_mut() = Some(pending_state);
        let _guard = PendingGuard::new(&self.state.pending);
        response_signal.reset();

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
        match embassy_time::with_timeout(self.timeout, response_signal.wait()).await {
            Ok(inner_result) => inner_result,
            Err(_) => Err(SdoError::Timeout),
        }
    }

    pub async fn send_nmt(&self, cs: u8) -> Result<(), TX::Error> {
        // cs: command specifier
        let nmt_id = StandardId::new(0x000).unwrap();
        let payload = [cs, self.node_id];
        let frame = FRAME::new(Id::Standard(nmt_id), &payload).unwrap();
        self.can_tx.lock().await.transmit(&frame).await
    }

    pub async fn send_nmt_start_node(&self) -> Result<(), TX::Error> {
        self.send_nmt(NMT_CS_START).await
    }

    pub async fn send_nmt_stop_node(&self) -> Result<(), TX::Error> {
        self.send_nmt(NMT_CS_STOP).await
    }

    pub async fn send_nmt_enter_pre_operational(&self) -> Result<(), TX::Error> {
        self.send_nmt(NMT_CS_ENTER_PRE_OPERATIONAL).await
    }

    pub async fn send_nmt_reset_node(&self) -> Result<(), TX::Error> {
        self.send_nmt(NMT_CS_RESET_NODE).await
    }

    pub async fn send_nmt_reset_communication(&self) -> Result<(), TX::Error> {
        self.send_nmt(NMT_CS_RESET_COMMUNICATION).await
    }

    pub async fn read_expedited(&self, index: u16, sub: u8) -> Result<u32, SdoError<TX::Error>> {
        let _guard = self.request_lock.lock().await;
        let result = self.read_expedited_locked(index, sub).await;
        self.abort_on_local_failure(index, sub, result).await
    }

    pub async fn write_expedited(
        &self,
        index: u16,
        sub: u8,
        data: &[u8],
    ) -> Result<(), SdoError<TX::Error>> {
        let _guard = self.request_lock.lock().await;
        let result = self.write_expedited_locked(index, sub, data).await;
        self.abort_on_local_failure(index, sub, result).await
    }

    /// Read an object of arbitrary size into `buf`.
    ///
    /// Handles both server reply types: an expedited response delivers up to
    /// 4 bytes directly, a segmented response is read segment by segment.
    /// Returns the number of bytes received.
    pub async fn read_segmented(
        &self,
        index: u16,
        sub: u8,
        buf: &mut [u8],
    ) -> Result<usize, SdoError<TX::Error>> {
        let _guard = self.request_lock.lock().await;
        let result = self.read_segmented_locked(index, sub, buf).await;
        self.abort_on_local_failure(index, sub, result).await
    }

    #[allow(dead_code)]
    pub async fn write_segmented(
        &self,
        index: u16,
        sub: u8,
        data: &[u8],
    ) -> Result<(), SdoError<TX::Error>> {
        let _guard = self.request_lock.lock().await;
        let result = self.write_segmented_locked(index, sub, data).await;
        self.abort_on_local_failure(index, sub, result).await
    }

    /// Read an object via block transfer into `buf`.
    ///
    /// Returns the number of bytes received.
    pub async fn read_block(
        &self,
        index: u16,
        sub: u8,
        buf: &mut [u8],
        request_crc_support: bool,
    ) -> Result<usize, SdoError<TX::Error>> {
        let _guard = self.request_lock.lock().await;
        let result = self
            .read_block_locked(index, sub, buf, request_crc_support)
            .await;
        self.abort_on_local_failure(index, sub, result).await
    }

    pub async fn write_block<S: StreamReader<TX::Error> + StreamSeeker<TX::Error>>(
        &self,
        index: u16,
        sub: u8,
        stream: &mut S,
        size: u32,
        request_crc_support: bool,
    ) -> Result<(), SdoError<TX::Error>> {
        let _guard = self.request_lock.lock().await;
        let result = self
            .write_block_locked(index, sub, stream, size, request_crc_support)
            .await;
        self.abort_on_local_failure(index, sub, result).await
    }

    #[deprecated(note = "use the synchronous `process_frame` instead")]
    pub async fn on_frame_received(&self, frame: FRAME) {
        self.process_frame(&frame);
    }

    /// Process a frame received on the bus. Frames that are not an SDO
    /// response for this node, or that cannot belong to the request in
    /// flight, are ignored. Synchronous: safe to call from any RX context.
    pub fn process_frame(&self, frame: &FRAME) {
        let expected_id = 0x580 + self.node_id as u16;
        let Id::Standard(id) = frame.id() else {
            return; // Not a standard frame
        };
        if id.as_raw() != expected_id {
            return; // Not an SDO response for this node
        }

        // Ignore frames that cannot belong to the pending request (stale or
        // duplicated responses): consuming the pending state for them would
        // turn the real response into a timeout.
        {
            let pending = self.state.pending.borrow();
            let Some(pending) = *pending else {
                // Frame arrived after a timeout but before the guard dropped
                return;
            };
            let Some(&command) = frame.data().first() else {
                return; // Empty frame is never a valid SDO response
            };
            if !crate::protocol::response_matches(pending, command) {
                return;
            }
        }

        // For BlockUploadActive, we peek instead of taking, to avoid race conditions.
        let mut is_block_upload_active = false;
        if let Some(Pending::BlockUploadActive) = *self.state.pending.borrow() {
            is_block_upload_active = true;
        }

        let pending_request = if !is_block_upload_active {
            self.state.pending.borrow_mut().take()
        } else {
            // It's a block upload segment, just peek at the state.
            // The state will be consumed later if it's the `last` segment.
            *self.state.pending.borrow()
        };

        let Some(pending_request) = pending_request else {
            // This can happen if a frame arrives after a timeout but before the guard is dropped
            return;
        };

        match parse_response(pending_request, frame.data()) {
            Err(err) => self.signal_error(pending_request, err.into_sdo_error()),
            Ok(outcome) => match outcome {
                ParseOutcome::UploadInit(init) => self.state.sig_upload_init.signal(Ok(init)),
                ParseOutcome::Ack => self.state.sig_ack.signal(Ok(())),
                ParseOutcome::Segment(segment) => self.state.sig_seg.signal(Ok(segment)),
                ParseOutcome::BlockInit(init) => self.state.sig_block_init.signal(Ok(init)),
                ParseOutcome::BlockAck(ack) => self.state.sig_block_ack.signal(Ok(ack)),
                ParseOutcome::BlockUploadInit(init) => {
                    self.state.sig_block_upload_init.signal(Ok(init))
                }
                ParseOutcome::BlockUploadSegment(segment) => {
                    if self
                        .state
                        .block_upload_seg_chan
                        .try_send(Ok(segment))
                        .is_err()
                    {
                        // This can happen if the server sends faster than the client task consumes.
                        // We drop the segment; the client task will eventually time out and abort.
                    }

                    // If it was the last segment of the whole transfer, consume the pending state.
                    if segment.last {
                        *self.state.pending.borrow_mut() = None;
                    }
                }
                ParseOutcome::BlockUploadEnd { crc, unused_bytes } => {
                    self.state.sig_block_upload_end.signal(Ok((crc, unused_bytes)))
                }
            },
        }
    }

    /// Send an initiate-upload request; the server decides whether the reply
    /// is expedited or announces a segmented transfer.
    async fn initiate_upload_locked(
        &self,
        index: u16,
        sub: u8,
    ) -> Result<UploadInit, SdoError<TX::Error>> {
        self.request_response(
            SdoRequest::UploadExpedited { index, sub },
            Pending::ExpeditedRead { index, sub },
            &self.state.sig_upload_init,
        )
        .await
    }

    async fn read_expedited_locked(&self, index: u16, sub: u8) -> Result<u32, SdoError<TX::Error>> {
        match self.initiate_upload_locked(index, sub).await? {
            UploadInit::Expedited { data, .. } => Ok(u32::from_le_bytes(data)),
            UploadInit::Segmented { .. } => Err(SdoError::NotExpedited),
        }
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
            &self.state.sig_ack,
        )
        .await
    }

    async fn read_segmented_locked(
        &self,
        index: u16,
        sub: u8,
        buf: &mut [u8],
    ) -> Result<usize, SdoError<TX::Error>> {
        match self.initiate_upload_locked(index, sub).await? {
            // The server is free to answer expedited if the object fits in
            // 4 bytes; deliver the data instead of desyncing the transfer.
            UploadInit::Expedited { data, len } => {
                if len > buf.len() {
                    return Err(SdoError::BufferSizeWrong);
                }
                buf[..len].copy_from_slice(&data[..len]);
                Ok(len)
            }

            UploadInit::Segmented { size } => {
                if let Some(size) = size
                    && size as usize > buf.len()
                {
                    return Err(SdoError::BufferSizeWrong);
                }

                // Initiation successful, start requesting segments
                let mut offset = 0usize;
                let mut toggle = false;

                loop {
                    let segment = self
                        .request_response(
                            SdoRequest::UploadSegment { toggle },
                            Pending::UploadSegment { toggle },
                            &self.state.sig_seg,
                        )
                        .await?;

                    if offset + segment.len > buf.len() {
                        return Err(SdoError::BufferSizeWrong);
                    }
                    buf[offset..offset + segment.len]
                        .copy_from_slice(&segment.data[..segment.len]);
                    offset += segment.len;

                    if segment.last {
                        break;
                    }

                    toggle = !toggle;
                }

                Ok(offset)
            }
        }
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
            &self.state.sig_ack,
        )
        .await?;

        // A zero-length download still requires one (empty) last segment,
        // otherwise the server waits for data forever.
        if data.is_empty() {
            return self
                .request_response(
                    SdoRequest::DownloadSegment {
                        toggle: false,
                        last: true,
                        data: &[],
                    },
                    Pending::DownloadSegment { toggle: false },
                    &self.state.sig_ack,
                )
                .await;
        }

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
                &self.state.sig_ack,
            )
            .await?;

            toggle = !toggle;
        }
        Ok(())
    }

    async fn read_block_locked(
        &self,
        index: u16,
        sub: u8,
        buf: &mut [u8],
        request_crc_support: bool,
    ) -> Result<usize, SdoError<TX::Error>> {
        const XMODEM: Crc<u16> = Crc::<u16>::new(&Algorithm {
            width: 16,
            poly: 0x1021,
            init: 0x0000,
            refin: false,
            refout: false,
            xorout: 0x0000,
            check: 0x31C3,
            residue: 0x0000,
        });
        let mut crc_digest = XMODEM.digest();

        // --- 1. Initiate Block Upload ---
        *self.state.pending.borrow_mut() = Some(Pending::BlockUploadInitiate { index, sub });
        let _guard = PendingGuard::new(&self.state.pending);
        self.state.sig_block_upload_init.reset();

        // The client proposes a block size. 127 is the max.
        let client_blksize: u8 = 127;

        self.send_initiate_block_upload(index, sub, client_blksize, request_crc_support)
            .await
            .map_err(SdoError::TxError)?;

        let BlockUploadInit {
            size,
            server_supports_crc,
        } = match embassy_time::with_timeout(self.timeout, self.state.sig_block_upload_init.wait())
            .await
        {
            Ok(res) => res?,
            Err(_) => return Err(SdoError::Timeout),
        };

        if let Some(s) = size
            && s as usize > buf.len()
        {
            return Err(SdoError::BufferSizeWrong);
        }
        let use_crc = request_crc_support && server_supports_crc;

        // --- 2. Start Upload ---
        self.send_start_block_upload()
            .await
            .map_err(SdoError::TxError)?;

        // --- 3. Main Loop: Receive sub-blocks ---
        let mut offset = 0usize;
        let mut transfer_complete = matches!(size, Some(0));

        // The final segment (c bit set) is held back: how many of its bytes
        // are valid is only known from the end-of-transfer frame's n field.
        let mut final_segment: Option<[u8; 7]> = None;

        // Clear any stale segments from a previous failed run
        while self.state.block_upload_seg_chan.try_receive().is_ok() {}

        while !transfer_complete {
            let mut last_received_seqno = 0;
            // --- Receive one sub-block ---
            for expected_seqno in 1..=client_blksize {
                *self.state.pending.borrow_mut() = Some(Pending::BlockUploadActive);

                let segment = match embassy_time::with_timeout(
                    self.timeout,
                    self.state.block_upload_seg_chan.receive(),
                )
                .await
                {
                    Ok(res) => res?,
                    Err(_) => return Err(SdoError::Timeout),
                };

                if segment.seqno != expected_seqno {
                    return Err(self.abort_transfer(index, sub, ABORT_SEQ_NUM_ERROR).await);
                }

                last_received_seqno = segment.seqno;

                if segment.last {
                    final_segment = Some(segment.data);
                    transfer_complete = true;
                    break;
                }

                // Intermediate segments always carry 7 data bytes.
                if offset + segment.len > buf.len() {
                    return Err(SdoError::BufferSizeWrong);
                }
                buf[offset..offset + segment.len].copy_from_slice(&segment.data[..segment.len]);

                if use_crc {
                    crc_digest.update(&buf[offset..offset + segment.len]);
                }

                offset += segment.len;
            }

            // --- 4. Acknowledge sub-block ---
            self.send_block_upload_ack(last_received_seqno, client_blksize)
                .await
                .map_err(SdoError::TxError)?;
        }

        // --- 5. Wait for End of Transfer frame from Server ---
        *self.state.pending.borrow_mut() = Some(Pending::BlockUploadEndWait);
        self.state.sig_block_upload_end.reset();

        let (server_crc, unused_bytes) =
            match embassy_time::with_timeout(self.timeout, self.state.sig_block_upload_end.wait())
                .await
            {
                Ok(res) => res?,
                Err(_) => return Err(SdoError::Timeout),
            };

        // --- 6. Commit the held-back final segment now that its valid
        // length is known from the end frame ---
        if let Some(data) = final_segment {
            let valid = 7usize.saturating_sub(unused_bytes as usize);
            if offset + valid > buf.len() {
                return Err(SdoError::BufferSizeWrong);
            }
            buf[offset..offset + valid].copy_from_slice(&data[..valid]);
            if use_crc {
                crc_digest.update(&buf[offset..offset + valid]);
            }
            offset += valid;
        }

        // If the server announced a size, the received byte count must match.
        if let Some(s) = size
            && offset != s as usize
        {
            return Err(SdoError::InvalidResponse);
        }

        // --- 7. Validate CRC and send final confirmation ---
        if use_crc {
            let client_crc = crc_digest.finalize();
            if client_crc != server_crc {
                return Err(self.abort_transfer(index, sub, ABORT_CRC_ERROR).await);
            }
        }

        self.send_end_block_upload_confirmation()
            .await
            .map_err(SdoError::TxError)?;

        Ok(offset)
    }

    async fn write_block_locked<S: StreamReader<TX::Error> + StreamSeeker<TX::Error>>(
        &self,
        index: u16,
        sub: u8,
        stream: &mut S,
        size: u32,
        request_crc_support: bool,
    ) -> Result<(), SdoError<TX::Error>> {
        const XMODEM: Crc<u16> = Crc::<u16>::new(&Algorithm {
            width: 16,
            poly: 0x1021,
            init: 0x0000,
            refin: false,
            refout: false,
            xorout: 0x0000,
            check: 0x31C3,
            residue: 0x0000,
        });
        let mut crc_digest = XMODEM.digest();

        // --- Initiate Block Download ---
        *self.state.pending.borrow_mut() = Some(Pending::BlockDownloadInitiate { index, sub });
        let _guard = PendingGuard::new(&self.state.pending);
        self.state.sig_block_init.reset();

        self.send_initiate_block_download(index, sub, size, request_crc_support)
            .await
            .map_err(SdoError::TxError)?;

        let BlockInit {
            mut blksize,
            server_supports_crc,
        } = match embassy_time::with_timeout(self.timeout, self.state.sig_block_init.wait()).await {
            Ok(res) => res?,
            Err(_) => return Err(SdoError::Timeout),
        };

        let use_crc = request_crc_support && server_supports_crc;

        if blksize == 0 || blksize > 127 {
            return Err(self.abort_transfer(index, sub, ABORT_INVALID_BLOCK_SIZE).await);
        }

        // --- Main Loop: Send sub-blocks ---
        let mut offset = 0usize;
        let mut chunk_buf = [0u8; 7];

        while offset < size as usize {
            // --- Send one sub-block ---
            let sub_block_start_offset = offset;
            let mut last_sent_seqno_in_block = 0;

            for seqno in 1..=blksize {
                let bytes_read = stream.read(&mut chunk_buf).await?;
                if bytes_read == 0 {
                    // The stream ended before the promised `size` was delivered. This is an error.
                    return Err(SdoError::StreamError);
                }
                let chunk = &chunk_buf[..bytes_read];

                if use_crc {
                    crc_digest.update(chunk);
                }

                offset += bytes_read;
                last_sent_seqno_in_block = seqno;
                let last_segment_of_transfer = offset == size as usize;

                self.send_block_download_segment(chunk, seqno, last_segment_of_transfer)
                    .await
                    .map_err(SdoError::TxError)?;

                if last_segment_of_transfer {
                    break;
                }
            }

            // If nothing was sent in this sub-block, break the main loop.
            if last_sent_seqno_in_block == 0 {
                break;
            }

            // --- Await sub-block acknowledgement ---
            'ack_loop: loop {
                *self.state.pending.borrow_mut() = Some(Pending::BlockDownloadAck);
                self.state.sig_block_ack.reset();

                let BlockAck {
                    ackseq,
                    next_blksize,
                } = match embassy_time::with_timeout(self.timeout, self.state.sig_block_ack.wait())
                    .await
                {
                    Ok(res) => res?,
                    Err(_) => return Err(SdoError::Timeout),
                };

                if ackseq > last_sent_seqno_in_block {
                    return Err(SdoError::InvalidResponse);
                }

                if ackseq == last_sent_seqno_in_block {
                    if next_blksize > 0 && next_blksize <= 127 {
                        blksize = next_blksize;
                    } else if next_blksize != 0 {
                        return Err(self.abort_transfer(index, sub, ABORT_INVALID_BLOCK_SIZE).await);
                    }
                    break 'ack_loop;
                } else {
                    // Retransmission needed
                    let retransmit_start_offset = sub_block_start_offset + (ackseq as usize * 7);

                    // Seek to the point where retransmission should start
                    stream
                        .seek(SeekFrom::Start(retransmit_start_offset as u64))
                        .await?;

                    let mut retransmit_buf = [0u8; 7];

                    for seqno_to_resend in (ackseq + 1)..=last_sent_seqno_in_block {
                        let bytes_read = stream.read(&mut retransmit_buf).await?;
                        if bytes_read == 0 {
                            // Stream ended prematurely during retransmission
                            return Err(SdoError::StreamError);
                        }
                        let chunk = &retransmit_buf[..bytes_read];

                        let retransmitted_bytes_count =
                            ((seqno_to_resend - ackseq - 1) as usize * 7) + bytes_read;
                        let current_retransmit_stream_pos =
                            retransmit_start_offset + retransmitted_bytes_count;

                        let is_last = current_retransmit_stream_pos == size as usize;

                        self.send_block_download_segment(chunk, seqno_to_resend, is_last)
                            .await
                            .map_err(SdoError::TxError)?;

                        if is_last {
                            break;
                        }
                    }
                }
            }
        }

        // --- End of Transfer ---
        let crc_val = if use_crc {
            crc_digest.finalize()
        } else {
            0x0000
        };

        let unused_bytes_in_last_segment = if size == 0 {
            7
        } else {
            let last_segment_len = (size as usize - 1) % 7 + 1;
            (7 - last_segment_len) as u8
        };

        *self.state.pending.borrow_mut() = Some(Pending::BlockDownloadEnd);
        self.state.sig_ack.reset();

        self.send_end_block_download(unused_bytes_in_last_segment, crc_val)
            .await
            .map_err(SdoError::TxError)?;

        match embassy_time::with_timeout(self.timeout, self.state.sig_ack.wait()).await {
            Ok(res) => res?,
            Err(_) => return Err(SdoError::Timeout),
        };

        Ok(())
    }

    fn signal_error(&self, pending: Pending, err: SdoError<TX::Error>) {
        match pending {
            Pending::ExpeditedRead { .. } => self.state.sig_upload_init.signal(Err(err)),
            Pending::ExpeditedWrite { .. }
            | Pending::SegmentedDownloadInit { .. }
            | Pending::DownloadSegment { .. }
            | Pending::BlockDownloadEnd => self.state.sig_ack.signal(Err(err)),
            Pending::UploadSegment { .. } => self.state.sig_seg.signal(Err(err)),
            Pending::BlockDownloadInitiate { .. } => self.state.sig_block_init.signal(Err(err)),
            Pending::BlockDownloadAck => self.state.sig_block_ack.signal(Err(err)),
            Pending::BlockUploadInitiate { .. } => {
                self.state.sig_block_upload_init.signal(Err(err))
            }
            Pending::BlockUploadActive => {
                if self.state.block_upload_seg_chan.try_send(Err(err)).is_err() {
                    // queue is full, dropping error.
                }
            }
            Pending::BlockUploadEndWait => self.state.sig_block_upload_end.signal(Err(err)),
        }
    }

    // ## --- HELPER SENDER FUNCTIONS --- ##

    /// Transmit an SDO request payload to the server (COB-ID 0x600 + node id).
    async fn send_payload(&self, payload: &[u8; 8]) -> Result<(), TX::Error> {
        let id = 0x600 + (self.node_id as u16);
        let frame = FRAME::new(Id::Standard(StandardId::new(id).unwrap()), payload).unwrap();
        self.can_tx.lock().await.transmit(&frame).await
    }

    /// Send an abort for the transfer on `index`/`sub` and return the
    /// matching error. Used where the client itself detects a protocol
    /// violation mid-transfer.
    async fn abort_transfer(&self, index: u16, sub: u8, code: u32) -> SdoError<TX::Error> {
        // Best effort: the abort must not mask the original error.
        let _ = self.send_payload(&encode_abort(index, sub, code)).await;
        SdoError::SdoAbort(code)
    }

    /// Transmit an abort for local failures so the server releases its
    /// transfer state (CiA 301 requires the giving-up peer to abort).
    ///
    /// Server-sent aborts (`SdoAbort` from a response) and TX failures are
    /// passed through: the former needs no reply, the latter cannot be sent.
    /// Locally raised `SdoAbort`s go through [`Self::abort_transfer`] instead.
    async fn abort_on_local_failure<T>(
        &self,
        index: u16,
        sub: u8,
        result: Result<T, SdoError<TX::Error>>,
    ) -> Result<T, SdoError<TX::Error>> {
        if let Err(err) = &result {
            let code = match err {
                SdoError::Timeout => ABORT_TIMEOUT,
                SdoError::InvalidResponse => ABORT_INVALID_CS,
                SdoError::BufferSizeWrong => ABORT_OUT_OF_MEMORY,
                // The server started a transfer we are not going to finish
                SdoError::NotExpedited => ABORT_GENERAL_ERROR,
                SdoError::StreamError => ABORT_GENERAL_ERROR,
                _ => return result,
            };
            let _ = self.send_payload(&encode_abort(index, sub, code)).await;
        }
        result
    }

    async fn send_sdo_upload_request(&self, index: u16, subindex: u8) -> Result<(), TX::Error> {
        self.send_payload(&encode_upload_request(index, subindex)).await
    }

    async fn send_sdo_download_request(
        &self,
        index: u16,
        subindex: u8,
        data: &[u8],
    ) -> Result<(), TX::Error> {
        self.send_payload(&encode_download_expedited(index, subindex, data)).await
    }

    async fn send_initiate_segmented_download(
        &self,
        index: u16,
        sub: u8,
        size: u32,
    ) -> Result<(), TX::Error> {
        self.send_payload(&encode_initiate_segmented_download(index, sub, size)).await
    }

    async fn send_sdo_download_segment(
        &self,
        segment_data: &[u8],
        toggle: bool,
        last: bool,
    ) -> Result<(), TX::Error> {
        self.send_payload(&encode_download_segment(segment_data, toggle, last)).await
    }

    async fn send_sdo_request_upload_segment(&self, toggle: bool) -> Result<(), TX::Error> {
        self.send_payload(&encode_upload_segment_request(toggle)).await
    }

    async fn send_initiate_block_download(
        &self,
        index: u16,
        sub: u8,
        size: u32,
        crc: bool,
    ) -> Result<(), TX::Error> {
        self.send_payload(&encode_initiate_block_download(index, sub, size, crc)).await
    }

    async fn send_block_download_segment(
        &self,
        segment_data: &[u8],
        seqno: u8,
        last_segment: bool,
    ) -> Result<(), TX::Error> {
        self.send_payload(&encode_block_download_segment(segment_data, seqno, last_segment))
            .await
    }

    async fn send_end_block_download(&self, unused_bytes: u8, crc: u16) -> Result<(), TX::Error> {
        self.send_payload(&encode_end_block_download(unused_bytes, crc)).await
    }

    async fn send_initiate_block_upload(
        &self,
        index: u16,
        sub: u8,
        blksize: u8,
        crc: bool,
    ) -> Result<(), TX::Error> {
        self.send_payload(&encode_initiate_block_upload(index, sub, blksize, crc)).await
    }

    async fn send_start_block_upload(&self) -> Result<(), TX::Error> {
        self.send_payload(&encode_start_block_upload()).await
    }

    async fn send_block_upload_ack(&self, ackseq: u8, blksize: u8) -> Result<(), TX::Error> {
        self.send_payload(&encode_block_upload_ack(ackseq, blksize)).await
    }

    async fn send_end_block_upload_confirmation(&self) -> Result<(), TX::Error> {
        self.send_payload(&encode_end_block_upload_confirmation()).await
    }
}
