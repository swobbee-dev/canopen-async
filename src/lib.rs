#![no_std]
#![allow(async_fn_in_trait)]

use core::cell::RefCell;
use crc::{Algorithm, Crc};
use embassy_sync::{blocking_mutex::raw::NoopRawMutex, mutex::Mutex, signal::Signal};
use embassy_time::Duration;
use embedded_can::{Frame, Id, StandardId, asynch::CanTx};

const ABORT_INVALID_BLOCK_SIZE: u32 = 0x05040002;
const ABORT_SEQ_NUM_ERROR: u32 = 0x05040003;
const ABORT_CRC_ERROR: u32 = 0x05040004;

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

struct RequestState<FRAME, TX: CanTx<Frame = FRAME>> {
    pending: RefCell<Option<Pending>>,
    sig_word: Signal<NoopRawMutex, Result<u32, SdoError<TX::Error>>>,
    sig_seg: Signal<NoopRawMutex, Result<Segment, SdoError<TX::Error>>>,
    sig_ack: Signal<NoopRawMutex, Result<(), SdoError<TX::Error>>>,
    sig_block_init: Signal<NoopRawMutex, Result<BlockInit, SdoError<TX::Error>>>,
    sig_block_ack: Signal<NoopRawMutex, Result<BlockAck, SdoError<TX::Error>>>,
    sig_block_upload_init: Signal<NoopRawMutex, Result<BlockUploadInit, SdoError<TX::Error>>>,
    sig_block_upload_seg: Signal<NoopRawMutex, Result<BlockUploadSegment, SdoError<TX::Error>>>,
    sig_block_upload_end: Signal<NoopRawMutex, Result<u16, SdoError<TX::Error>>>,
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
    BlockDownloadInitiate { index: u16, sub: u8 },
    BlockDownloadAck,
    BlockDownloadEnd,
    BlockUploadInitiate { index: u16, sub: u8 },
    BlockUploadActive,
    BlockUploadEndWait,
}

#[derive(Copy, Clone)]
pub struct Segment {
    last: bool,
    len: usize,
    data: [u8; 7],
}

pub struct BlockInit {
    blksize: u8,
    server_supports_crc: bool,
}

pub struct BlockAck {
    ackseq: u8,
    next_blksize: u8,
}

pub struct BlockUploadInit {
    size: u32,
    server_supports_crc: bool,
}

#[derive(Copy, Clone)]
pub struct BlockUploadSegment {
    last: bool,
    seqno: u8,
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
                sig_word: Signal::new(),
                sig_seg: Signal::new(),
                sig_ack: Signal::new(),
                sig_block_init: Signal::new(),
                sig_block_ack: Signal::new(),
                sig_block_upload_init: Signal::new(),
                sig_block_upload_seg: Signal::new(),
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

    pub async fn send_nmt_reset_node(&self) -> Result<(), TX::Error> {
        const NMT_CMD_RESET_NODE: u8 = 0x81;
        let nmt_id = StandardId::new(0x000).unwrap();
        let payload = [NMT_CMD_RESET_NODE, self.node_id];
        let frame = FRAME::new(Id::Standard(nmt_id), &payload).unwrap();
        self.can_tx.lock().await.transmit(&frame).await
    }

    pub async fn read_expedited(&self, index: u16, sub: u8) -> Result<u32, SdoError<TX::Error>> {
        let _guard = self.request_lock.lock().await;
        self.read_expedited_locked(index, sub).await
    }

    pub async fn write_expedited(
        &self,
        index: u16,
        sub: u8,
        data: &[u8],
    ) -> Result<(), SdoError<TX::Error>> {
        let _guard = self.request_lock.lock().await;
        self.write_expedited_locked(index, sub, data).await
    }

    pub async fn read_segmented(
        &self,
        index: u16,
        sub: u8,
        buf: &mut [u8],
    ) -> Result<(), SdoError<TX::Error>> {
        let _guard = self.request_lock.lock().await;
        self.read_segmented_locked(index, sub, buf).await
    }

    #[allow(dead_code)]
    pub async fn write_segmented(
        &self,
        index: u16,
        sub: u8,
        data: &[u8],
    ) -> Result<(), SdoError<TX::Error>> {
        let _guard = self.request_lock.lock().await;
        self.write_segmented_locked(index, sub, data).await
    }

    pub async fn read_block(
        &self,
        index: u16,
        sub: u8,
        buf: &mut [u8],
        request_crc_support: bool,
    ) -> Result<(), SdoError<TX::Error>> {
        let _guard = self.request_lock.lock().await;
        self.read_block_locked(index, sub, buf, request_crc_support)
            .await
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
        self.write_block_locked(index, sub, stream, size, request_crc_support)
            .await
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
                    self.state.sig_word.signal(Err(SdoError::InvalidResponse));
                    return;
                }

                match command {
                    // Expedited Upload Response (e.g. 0x43, 0x47, 0x4B, 0x4F)
                    0x43 | 0x47 | 0x4B | 0x4F => {
                        let n_unused = ((command & 0x0C) >> 2) as usize;
                        let data_len = 4 - n_unused;
                        let mut bytes = [0u8; 4];
                        bytes[..data_len].copy_from_slice(&frame.data()[4..4 + data_len]);
                        self.state.sig_word.signal(Ok(u32::from_le_bytes(bytes)));
                    }
                    // Segmented Upload Initiation Response
                    0x41 => {
                        let size = u32::from_le_bytes(frame.data()[4..8].try_into().unwrap());
                        self.state.sig_word.signal(Ok(size));
                    }
                    // Any other command is invalid for this state.
                    _ => {
                        self.state.sig_word.signal(Err(SdoError::InvalidResponse));
                    }
                }
            }

            Pending::ExpeditedWrite { index, sub }
            | Pending::SegmentedDownloadInit { index, sub } => {
                let response_index = u16::from_le_bytes(frame.data()[1..3].try_into().unwrap());
                let response_sub = frame.data()[3];
                if response_index != index || response_sub != sub {
                    self.state.sig_ack.signal(Err(SdoError::InvalidResponse));
                    return;
                }

                // The only valid response is a download confirmation.
                if command == 0x60 {
                    self.state.sig_ack.signal(Ok(()));
                } else {
                    self.state.sig_ack.signal(Err(SdoError::InvalidResponse));
                }
            }

            Pending::UploadSegment { toggle } => {
                // Segment Upload Response: cs = 000t nnnc b
                if (command & 0xE0) == 0x00 {
                    let response_toggle = (command & 0x10) != 0;
                    if response_toggle != toggle {
                        self.state.sig_seg.signal(Err(SdoError::InvalidResponse));
                        return;
                    }

                    let last = (command & 0x01) != 0;
                    let n_unused = ((command & 0x0E) >> 1) as usize;
                    let len = 7 - n_unused;
                    let mut data = [0u8; 7];
                    data[..len].copy_from_slice(&frame.data()[1..1 + len]);
                    let segment = Segment { last, len, data };
                    self.state.sig_seg.signal(Ok(segment));
                } else {
                    self.state.sig_seg.signal(Err(SdoError::InvalidResponse));
                }
            }

            Pending::DownloadSegment { toggle } => {
                // Download Segment Response: cs = 001t 0000 b
                if (command & 0b1110_1111) == 0b0010_0000 {
                    let response_toggle = (command & 0b0001_0000) != 0;
                    if response_toggle == toggle {
                        self.state.sig_ack.signal(Ok(()));
                    } else {
                        self.state.sig_ack.signal(Err(SdoError::InvalidResponse));
                    }
                } else {
                    self.state.sig_ack.signal(Err(SdoError::InvalidResponse));
                }
            }
            Pending::BlockDownloadInitiate { index, sub } => {
                // Response to initiate block download: cs = 10100r00b
                if (command & 0b1111_1011) == 0b1010_0000 {
                    let response_index = u16::from_le_bytes(frame.data()[1..3].try_into().unwrap());
                    let response_sub = frame.data()[3];
                    if response_index != index || response_sub != sub {
                        self.state
                            .sig_block_init
                            .signal(Err(SdoError::InvalidResponse));
                        return;
                    }
                    let blksize = frame.data()[4];
                    let server_supports_crc = (command & 0b0000_0100) != 0;

                    self.state.sig_block_init.signal(Ok(BlockInit {
                        blksize,
                        server_supports_crc,
                    }));
                } else {
                    self.state
                        .sig_block_init
                        .signal(Err(SdoError::InvalidResponse));
                }
            }

            Pending::BlockDownloadAck => {
                // Response to sub-block: cs = 10100010b
                if command == 0xA2 {
                    // 10100010b
                    let ackseq = frame.data()[1];
                    let next_blksize = frame.data()[2];
                    self.state.sig_block_ack.signal(Ok(BlockAck {
                        ackseq,
                        next_blksize,
                    }));
                } else {
                    self.state
                        .sig_block_ack
                        .signal(Err(SdoError::InvalidResponse));
                }
            }

            Pending::BlockDownloadEnd => {
                // Response to end download: cs = 10100001b
                if command == 0xA1 {
                    // 10100001b
                    self.state.sig_ack.signal(Ok(()));
                } else {
                    self.state.sig_ack.signal(Err(SdoError::InvalidResponse));
                }
            }

            Pending::BlockUploadInitiate { index, sub } => {
                // Response to initiate block upload: cs = 11000rs0b
                if (command & 0b1111_1001) == 0b1100_0000 {
                    let response_index = u16::from_le_bytes(frame.data()[1..3].try_into().unwrap());
                    let response_sub = frame.data()[3];
                    if response_index != index || response_sub != sub {
                        self.state
                            .sig_block_upload_init
                            .signal(Err(SdoError::InvalidResponse));
                        return;
                    }

                    let server_supports_crc = (command & 0b0000_0100) != 0;
                    let size_indicated = (command & 0b0000_0010) != 0;

                    let size = if size_indicated {
                        u32::from_le_bytes(frame.data()[4..8].try_into().unwrap())
                    } else {
                        0
                    };

                    self.state.sig_block_upload_init.signal(Ok(BlockUploadInit {
                        size,
                        server_supports_crc,
                    }));
                } else {
                    self.state
                        .sig_block_upload_init
                        .signal(Err(SdoError::InvalidResponse));
                }
            }

            Pending::BlockUploadActive => {
                // Must be an upload segment: cs = c nnnnnnnb
                let seqno = command & 0x7F;
                if seqno > 0 && seqno <= 127 {
                    let last = (command & 0x80) != 0;
                    let len = 7;
                    let mut data = [0u8; 7];
                    data.copy_from_slice(&frame.data()[1..8]);

                    let segment = BlockUploadSegment {
                        last,
                        seqno,
                        len,
                        data,
                    };

                    self.state.sig_block_upload_seg.signal(Ok(segment));
                } else {
                    self.state
                        .sig_block_upload_seg
                        .signal(Err(SdoError::InvalidResponse));
                }
            }

            Pending::BlockUploadEndWait => {
                // Must be the end of upload frame from server: cs = 110nnn01b
                if (command & 0b11100011) == 0b11000001 {
                    let crc = u16::from_le_bytes(frame.data()[1..3].try_into().unwrap());
                    self.state.sig_block_upload_end.signal(Ok(crc));
                } else {
                    self.state
                        .sig_block_upload_end
                        .signal(Err(SdoError::InvalidResponse));
                }
            }
        };
    }

    async fn read_expedited_locked(&self, index: u16, sub: u8) -> Result<u32, SdoError<TX::Error>> {
        self.request_response(
            SdoRequest::UploadExpedited { index, sub },
            Pending::ExpeditedRead { index, sub },
            &self.state.sig_word,
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
            &self.state.sig_ack,
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
                    &self.state.sig_seg,
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
            &self.state.sig_ack,
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

        if size as usize > buf.len() {
            return Err(SdoError::BufferSizeWrong);
        }

        let use_crc = request_crc_support && server_supports_crc;

        // --- 2. Start Upload ---
        self.send_start_block_upload()
            .await
            .map_err(SdoError::TxError)?;

        // --- 3. Main Loop: Receive sub-blocks ---
        let mut offset = 0usize;
        let mut transfer_complete = size == 0;

        while !transfer_complete {
            let mut last_received_seqno = 0;

            // --- Receive one sub-block ---
            for expected_seqno in 1..=client_blksize {
                *self.state.pending.borrow_mut() = Some(Pending::BlockUploadActive);
                self.state.sig_block_upload_seg.reset();

                let segment = match embassy_time::with_timeout(
                    self.timeout,
                    self.state.sig_block_upload_seg.wait(),
                )
                .await
                {
                    Ok(res) => res?,
                    Err(_) => return Err(SdoError::Timeout),
                };

                if segment.seqno != expected_seqno {
                    return Err(SdoError::SdoAbort(ABORT_SEQ_NUM_ERROR));
                }

                let data_len = if size > 0 && (offset + segment.len) > size as usize {
                    size as usize - offset
                } else {
                    // When size is unknown (0), we must rely on the segment data length
                    segment.len
                };

                if offset + data_len > buf.len() {
                    return Err(SdoError::BufferSizeWrong);
                }

                buf[offset..offset + data_len].copy_from_slice(&segment.data[..data_len]);

                if use_crc {
                    crc_digest.update(&buf[offset..offset + data_len]);
                }

                offset += data_len;
                last_received_seqno = segment.seqno;

                if segment.last {
                    transfer_complete = true;
                    break; // Exit sub-block loop
                }
                if offset as u32 >= size {
                    transfer_complete = true;
                    break;
                }
            }

            // --- 4. Acknowledge sub-block ---
            self.send_block_upload_ack(last_received_seqno, client_blksize)
                .await
                .map_err(SdoError::TxError)?;
        }

        // --- 5. Wait for End of Transfer frame from Server ---
        *self.state.pending.borrow_mut() = Some(Pending::BlockUploadEndWait);
        self.state.sig_block_upload_end.reset();

        let server_crc =
            match embassy_time::with_timeout(self.timeout, self.state.sig_block_upload_end.wait())
                .await
            {
                Ok(res) => res?,
                Err(_) => return Err(SdoError::Timeout),
            };

        // --- 6. Validate CRC and send final confirmation ---
        if use_crc {
            let client_crc = crc_digest.finalize();
            if client_crc != server_crc {
                return Err(SdoError::SdoAbort(ABORT_CRC_ERROR));
            }
        }

        self.send_end_block_upload_confirmation()
            .await
            .map_err(SdoError::TxError)?;

        Ok(())
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
            return Err(SdoError::SdoAbort(ABORT_INVALID_BLOCK_SIZE));
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
                        return Err(SdoError::SdoAbort(ABORT_INVALID_BLOCK_SIZE));
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

                        // Calculate the "current" offset for the purpose of checking if this is the last segment
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
            Pending::ExpeditedRead { .. } => self.state.sig_word.signal(Err(err)),
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
            Pending::BlockUploadActive => self.state.sig_block_upload_seg.signal(Err(err)),
            Pending::BlockUploadEndWait => self.state.sig_block_upload_end.signal(Err(err)),
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
        self.can_tx.lock().await.transmit(&frame).await
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
        self.can_tx.lock().await.transmit(&frame).await
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
        self.can_tx.lock().await.transmit(&frame).await
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
        self.can_tx.lock().await.transmit(&frame).await
    }

    async fn send_sdo_request_upload_segment(&self, toggle: bool) -> Result<(), TX::Error> {
        let id = 0x600 + (self.node_id as u16);
        let mut payload = [0u8; 8];
        // Upload SDO Segment Request: 011t 0000 b
        payload[0] = 0x60 | if toggle { 0x10 } else { 0x00 };
        let frame = FRAME::new(Id::Standard(StandardId::new(id).unwrap()), &payload).unwrap();
        self.can_tx.lock().await.transmit(&frame).await
    }

    async fn send_initiate_block_download(
        &self,
        index: u16,
        sub: u8,
        size: u32,
        crc: bool,
    ) -> Result<(), TX::Error> {
        let id = 0x600 + (self.node_id as u16);
        let mut payload = [0u8; 8];
        // Initiate block download CS: 11000rs0b
        let mut cs = 0b1100_0000;
        if crc {
            cs |= 0b0000_0100; // r bit (CRC support)
        }
        cs |= 0b0000_0010; // s bit (size indicated)

        payload[0] = cs;
        payload[1..3].copy_from_slice(&index.to_le_bytes());
        payload[3] = sub;
        payload[4..8].copy_from_slice(&size.to_le_bytes());

        let frame = FRAME::new(Id::Standard(StandardId::new(id).unwrap()), &payload).unwrap();
        self.can_tx.lock().await.transmit(&frame).await
    }

    async fn send_block_download_segment(
        &self,
        segment_data: &[u8],
        seqno: u8,
        last_segment: bool,
    ) -> Result<(), TX::Error> {
        let id = 0x600 + (self.node_id as u16);
        let mut payload = [0u8; 8];

        // Download block segment CS: cnnnnnnnb
        let mut cs = seqno;
        if last_segment {
            cs |= 0x80; // c bit
        }

        payload[0] = cs;
        payload[1..1 + segment_data.len()].copy_from_slice(segment_data);

        let frame = FRAME::new(Id::Standard(StandardId::new(id).unwrap()), &payload).unwrap();
        self.can_tx.lock().await.transmit(&frame).await
    }

    async fn send_end_block_download(&self, unused_bytes: u8, crc: u16) -> Result<(), TX::Error> {
        let id = 0x600 + (self.node_id as u16);
        let mut payload = [0u8; 8];
        // End block download CS: 110nnn01b
        let cs = 0b1100_0001 | (unused_bytes << 2);

        payload[0] = cs;
        payload[1..3].copy_from_slice(&crc.to_le_bytes());

        let frame = FRAME::new(Id::Standard(StandardId::new(id).unwrap()), &payload).unwrap();
        self.can_tx.lock().await.transmit(&frame).await
    }

    async fn send_initiate_block_upload(
        &self,
        index: u16,
        sub: u8,
        blksize: u8,
        crc: bool,
    ) -> Result<(), TX::Error> {
        let id = 0x600 + (self.node_id as u16);
        let mut payload = [0u8; 8];
        // Initiate block upload CS: 10100r00b
        let mut cs = 0b1010_0000;
        if crc {
            cs |= 0b0000_0100; // r bit (CRC support)
        }

        payload[0] = cs;
        payload[1..3].copy_from_slice(&index.to_le_bytes());
        payload[3] = sub;
        payload[4] = blksize;

        let frame = FRAME::new(Id::Standard(StandardId::new(id).unwrap()), &payload).unwrap();
        self.can_tx.lock().await.transmit(&frame).await
    }

    async fn send_start_block_upload(&self) -> Result<(), TX::Error> {
        let id = 0x600 + (self.node_id as u16);
        let mut payload = [0u8; 8];
        // Start upload CS: 10100011b
        payload[0] = 0xA3;

        let frame = FRAME::new(Id::Standard(StandardId::new(id).unwrap()), &payload).unwrap();
        self.can_tx.lock().await.transmit(&frame).await
    }

    async fn send_block_upload_ack(&self, ackseq: u8, blksize: u8) -> Result<(), TX::Error> {
        let id = 0x600 + (self.node_id as u16);
        let mut payload = [0u8; 8];
        // Upload sub-block response CS: 10100010b
        payload[0] = 0xA2;
        payload[1] = ackseq;
        payload[2] = blksize;

        let frame = FRAME::new(Id::Standard(StandardId::new(id).unwrap()), &payload).unwrap();
        self.can_tx.lock().await.transmit(&frame).await
    }

    async fn send_end_block_upload_confirmation(&self) -> Result<(), TX::Error> {
        let id = 0x600 + (self.node_id as u16);
        let mut payload = [0u8; 8];
        // Upload end response CS: 10100001b
        payload[0] = 0xA1;

        let frame = FRAME::new(Id::Standard(StandardId::new(id).unwrap()), &payload).unwrap();
        self.can_tx.lock().await.transmit(&frame).await
    }
}
