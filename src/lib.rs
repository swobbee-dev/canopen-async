#![no_std]

use core::cell::RefCell;
use crc::{Algorithm, Crc};
use embassy_sync::{blocking_mutex::raw::NoopRawMutex, mutex::Mutex, signal::Signal};
use embassy_time::Duration;
use embedded_can::{Frame, Id, StandardId, asynch::CanTx};

const ABORT_INVALID_BLOCK_SIZE: u32 = 0x05040002;

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

    #[allow(dead_code)]
    pub async fn read_block(
        &self,
        index: u16,
        sub: u8,
        buf: &mut [u8],
    ) -> Result<(), SdoError<TX::Error>> {
        let _guard = self.request_lock.lock().await;
        self.read_block_locked(index, sub, buf).await
    }

    pub async fn write_block(
        &self,
        index: u16,
        sub: u8,
        data: &[u8],
        request_crc_support: bool,
    ) -> Result<(), SdoError<TX::Error>> {
        let _guard = self.request_lock.lock().await;
        self.write_block_locked(index, sub, data, request_crc_support)
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
        _index: u16,
        _sub: u8,
        _buf: &mut [u8],
    ) -> Result<(), SdoError<TX::Error>> {
        unimplemented!("SDO Block Read is not yet implemented");
    }

    async fn write_block_locked(
        &self,
        index: u16,
        sub: u8,
        data: &[u8],
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

        self.send_initiate_block_download(index, sub, data.len() as u32, request_crc_support)
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

        while offset < data.len() {
            // --- Send one sub-block ---
            let sub_block_start_offset = offset;
            let mut last_sent_seqno_in_block = 0;

            for seqno in 1..=blksize {
                let chunk_end = (offset + 7).min(data.len());
                let chunk = &data[offset..chunk_end];

                if use_crc {
                    crc_digest.update(chunk);
                }

                let last_segment_of_transfer = chunk_end == data.len();

                self.send_block_download_segment(chunk, seqno, last_segment_of_transfer)
                    .await
                    .map_err(SdoError::TxError)?;

                offset = chunk_end;
                last_sent_seqno_in_block = seqno;

                if last_segment_of_transfer {
                    break;
                }
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
                    let mut retransmit_offset = retransmit_start_offset;

                    for i in 0..(last_sent_seqno_in_block - ackseq) {
                        let retransmit_seqno = ackseq + 1 + i;
                        let chunk_end = (retransmit_offset + 7).min(data.len());
                        let chunk = &data[retransmit_offset..chunk_end];
                        let is_last = chunk_end == data.len();

                        self.send_block_download_segment(chunk, retransmit_seqno, is_last)
                            .await
                            .map_err(SdoError::TxError)?;

                        retransmit_offset = chunk_end;
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

        let unused_bytes_in_last_segment = if data.is_empty() {
            7
        } else {
            let last_segment_len = (data.len() - 1) % 7 + 1;
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
}
