//! Pure CANopen SDO frame encoding and response parsing (CiA 301, section 7.2.4).
//!
//! Everything in this module is synchronous and free of side effects: encoding
//! returns payload arrays and parsing maps a response payload plus the pending
//! request state to a [`ParseOutcome`]. This keeps the protocol logic
//! unit-testable without an executor or a CAN transport. Transmission,
//! signalling and timeouts live in [`SdoClient`](crate::SdoClient).

use crate::SdoError;

pub(crate) const ABORT_INVALID_BLOCK_SIZE: u32 = 0x05040002;
pub(crate) const ABORT_SEQ_NUM_ERROR: u32 = 0x05040003;
pub(crate) const ABORT_CRC_ERROR: u32 = 0x05040004;

/// The request the client is currently waiting on; determines how a response
/// payload is interpreted and which waiter an outcome or error is routed to.
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub(crate) enum Pending {
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
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Segment {
    pub(crate) last: bool,
    pub(crate) len: usize,
    pub(crate) data: [u8; 7],
}

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct BlockInit {
    pub(crate) blksize: u8,
    pub(crate) server_supports_crc: bool,
}

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct BlockAck {
    pub(crate) ackseq: u8,
    pub(crate) next_blksize: u8,
}

#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct BlockUploadInit {
    pub(crate) size: Option<u32>,
    pub(crate) server_supports_crc: bool,
}

#[derive(Copy, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct BlockUploadSegment {
    pub(crate) last: bool,
    pub(crate) seqno: u8,
    pub(crate) len: usize,
    pub(crate) data: [u8; 7],
}

/// Server's answer to an initiate-upload request (CiA 301, 7.2.4.3.6).
///
/// The server chooses the transfer type: small objects come back expedited
/// (data in the initiate response itself), larger ones announce a segmented
/// transfer. Both are legal answers to the same request, so callers must
/// handle both.
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub(crate) enum UploadInit {
    /// Expedited response: `data[..len]` is the object value. Bytes beyond
    /// `len` are zeroed. If the server did not indicate a size (cs `0x42`),
    /// `len` is 4.
    Expedited { data: [u8; 4], len: usize },
    /// Segmented transfer announced; `size` is the total byte count if the
    /// server indicated one.
    Segmented { size: Option<u32> },
}

/// Successful interpretation of a response payload for a [`Pending`] request.
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub(crate) enum ParseOutcome {
    /// Response to an initiate-upload request.
    UploadInit(UploadInit),
    /// Successful acknowledge without payload.
    Ack,
    /// One upload segment.
    Segment(Segment),
    BlockInit(BlockInit),
    BlockAck(BlockAck),
    BlockUploadInit(BlockUploadInit),
    BlockUploadSegment(BlockUploadSegment),
    BlockUploadEnd { crc: u16 },
}

/// Protocol-level failure while interpreting a response payload.
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub(crate) enum ParseError {
    InvalidResponse,
    Abort(u32),
}

impl ParseError {
    pub(crate) fn into_sdo_error<E>(self) -> SdoError<E> {
        match self {
            ParseError::InvalidResponse => SdoError::InvalidResponse,
            ParseError::Abort(code) => SdoError::SdoAbort(code),
        }
    }
}

/// Interpret a response payload in the context of the pending request.
pub(crate) fn parse_response(pending: Pending, data: &[u8]) -> Result<ParseOutcome, ParseError> {
    if data.len() != 8 {
        return Err(ParseError::InvalidResponse);
    }

    let command = data[0];
    if command == 0x80 {
        // SDO Abort
        let abort_code = u32::from_le_bytes(data[4..8].try_into().unwrap());
        return Err(ParseError::Abort(abort_code));
    }

    match pending {
        Pending::ExpeditedRead { index, sub } => {
            check_multiplexer(data, index, sub)?;

            // Initiate upload response: cs = 010_0_nn_e_s
            match command {
                // Expedited (e=1). With size indicated (s=1) nn counts the
                // unused bytes; without (0x42, s=0) all 4 bytes may be valid.
                0x42 | 0x43 | 0x47 | 0x4B | 0x4F => {
                    let size_indicated = (command & 0x01) != 0;
                    let data_len = if size_indicated {
                        4 - ((command & 0x0C) >> 2) as usize
                    } else {
                        4
                    };
                    let mut bytes = [0u8; 4];
                    bytes[..data_len].copy_from_slice(&data[4..4 + data_len]);
                    Ok(ParseOutcome::UploadInit(UploadInit::Expedited {
                        data: bytes,
                        len: data_len,
                    }))
                }
                // Segmented (e=0), with (0x41) or without (0x40) total size
                0x41 => {
                    let size = u32::from_le_bytes(data[4..8].try_into().unwrap());
                    Ok(ParseOutcome::UploadInit(UploadInit::Segmented { size: Some(size) }))
                }
                0x40 => Ok(ParseOutcome::UploadInit(UploadInit::Segmented { size: None })),
                _ => Err(ParseError::InvalidResponse),
            }
        }

        Pending::ExpeditedWrite { index, sub } | Pending::SegmentedDownloadInit { index, sub } => {
            check_multiplexer(data, index, sub)?;

            if command == 0x60 {
                Ok(ParseOutcome::Ack)
            } else {
                Err(ParseError::InvalidResponse)
            }
        }

        Pending::UploadSegment { toggle } => {
            // Segment Upload Response: cs = 000t nnnc b
            if (command & 0xE0) == 0x00 {
                let response_toggle = (command & 0x10) != 0;
                if response_toggle != toggle {
                    return Err(ParseError::InvalidResponse);
                }

                let last = (command & 0x01) != 0;
                let n_unused = ((command & 0x0E) >> 1) as usize;
                let len = 7 - n_unused;
                let mut segment_data = [0u8; 7];
                segment_data[..len].copy_from_slice(&data[1..1 + len]);
                Ok(ParseOutcome::Segment(Segment {
                    last,
                    len,
                    data: segment_data,
                }))
            } else {
                Err(ParseError::InvalidResponse)
            }
        }

        Pending::DownloadSegment { toggle } => {
            // Download Segment Response: cs = 001t 0000 b
            if (command & 0b1110_1111) == 0b0010_0000 {
                let response_toggle = (command & 0b0001_0000) != 0;
                if response_toggle == toggle {
                    Ok(ParseOutcome::Ack)
                } else {
                    Err(ParseError::InvalidResponse)
                }
            } else {
                Err(ParseError::InvalidResponse)
            }
        }

        Pending::BlockDownloadInitiate { index, sub } => {
            // Response to initiate block download: cs = 10100r00b
            if (command & 0b1111_1011) == 0b1010_0000 {
                check_multiplexer(data, index, sub)?;
                let blksize = data[4];
                let server_supports_crc = (command & 0b0000_0100) != 0;

                Ok(ParseOutcome::BlockInit(BlockInit {
                    blksize,
                    server_supports_crc,
                }))
            } else {
                Err(ParseError::InvalidResponse)
            }
        }

        Pending::BlockDownloadAck => {
            // Response to sub-block: cs = 10100010b
            if command == 0xA2 {
                Ok(ParseOutcome::BlockAck(BlockAck {
                    ackseq: data[1],
                    next_blksize: data[2],
                }))
            } else {
                Err(ParseError::InvalidResponse)
            }
        }

        Pending::BlockDownloadEnd => {
            // Response to end download: cs = 10100001b
            if command == 0xA1 {
                Ok(ParseOutcome::Ack)
            } else {
                Err(ParseError::InvalidResponse)
            }
        }

        Pending::BlockUploadInitiate { index, sub } => {
            // Response to initiate block upload: cs = 11000rs0b
            if (command & 0b1111_1001) == 0b1100_0000 {
                check_multiplexer(data, index, sub)?;

                let server_supports_crc = (command & 0b0000_0100) != 0;
                let size_indicated = (command & 0b0000_0010) != 0;

                let size = if size_indicated {
                    Some(u32::from_le_bytes(data[4..8].try_into().unwrap()))
                } else {
                    None
                };

                Ok(ParseOutcome::BlockUploadInit(BlockUploadInit {
                    size,
                    server_supports_crc,
                }))
            } else {
                Err(ParseError::InvalidResponse)
            }
        }

        Pending::BlockUploadActive => {
            let seqno = command & 0x7F;
            if seqno > 0 {
                let last = (command & 0x80) != 0;
                let mut segment_data = [0u8; 7];
                segment_data.copy_from_slice(&data[1..8]);

                Ok(ParseOutcome::BlockUploadSegment(BlockUploadSegment {
                    last,
                    seqno,
                    len: 7,
                    data: segment_data,
                }))
            } else {
                Err(ParseError::InvalidResponse)
            }
        }

        Pending::BlockUploadEndWait => {
            // Must be the end of upload frame from server: cs = 110nnn01b
            if (command & 0b11100011) == 0b11000001 {
                let crc = u16::from_le_bytes(data[1..3].try_into().unwrap());
                Ok(ParseOutcome::BlockUploadEnd { crc })
            } else {
                Err(ParseError::InvalidResponse)
            }
        }
    }
}

/// Validate the index/sub-index echoed by the server against the request.
fn check_multiplexer(data: &[u8], index: u16, sub: u8) -> Result<(), ParseError> {
    let response_index = u16::from_le_bytes(data[1..3].try_into().unwrap());
    let response_sub = data[3];
    if response_index != index || response_sub != sub {
        Err(ParseError::InvalidResponse)
    } else {
        Ok(())
    }
}

// ## --- REQUEST ENCODING --- ##

pub(crate) fn encode_upload_request(index: u16, sub: u8) -> [u8; 8] {
    let mut payload = [0u8; 8];
    payload[0] = 0x40; // Initiate Upload Request
    payload[1..3].copy_from_slice(&index.to_le_bytes());
    payload[3] = sub;
    payload
}

/// Expedited download request; `data` must be 1..=4 bytes.
pub(crate) fn encode_download_expedited(index: u16, sub: u8, data: &[u8]) -> [u8; 8] {
    let mut payload = [0u8; 8];
    let len = data.len();
    let n = 4 - len;
    // Expedited download CS: 001_0_nn_11 b
    payload[0] = 0b0010_0011 | ((n as u8) << 2);
    payload[1..3].copy_from_slice(&index.to_le_bytes());
    payload[3] = sub;
    payload[4..4 + len].copy_from_slice(data);
    payload
}

pub(crate) fn encode_initiate_segmented_download(index: u16, sub: u8, size: u32) -> [u8; 8] {
    let mut payload = [0u8; 8];
    payload[0] = 0x21; // Initiate Segmented Download, size is indicated
    payload[1..3].copy_from_slice(&index.to_le_bytes());
    payload[3] = sub;
    payload[4..8].copy_from_slice(&size.to_le_bytes());
    payload
}

/// Download segment; `segment_data` must be at most 7 bytes.
pub(crate) fn encode_download_segment(segment_data: &[u8], toggle: bool, last: bool) -> [u8; 8] {
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
    payload
}

pub(crate) fn encode_upload_segment_request(toggle: bool) -> [u8; 8] {
    let mut payload = [0u8; 8];
    // Upload SDO Segment Request: 011t 0000 b
    payload[0] = 0x60 | if toggle { 0x10 } else { 0x00 };
    payload
}

pub(crate) fn encode_initiate_block_download(index: u16, sub: u8, size: u32, crc: bool) -> [u8; 8] {
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
    payload
}

/// Block download segment; `segment_data` must be at most 7 bytes.
pub(crate) fn encode_block_download_segment(segment_data: &[u8], seqno: u8, last_segment: bool) -> [u8; 8] {
    let mut payload = [0u8; 8];
    // Download block segment CS: cnnnnnnnb
    let mut cs = seqno;
    if last_segment {
        cs |= 0x80; // c bit
    }
    payload[0] = cs;
    payload[1..1 + segment_data.len()].copy_from_slice(segment_data);
    payload
}

pub(crate) fn encode_end_block_download(unused_bytes: u8, crc: u16) -> [u8; 8] {
    let mut payload = [0u8; 8];
    // End block download CS: 110nnn01b
    payload[0] = 0b1100_0001 | (unused_bytes << 2);
    payload[1..3].copy_from_slice(&crc.to_le_bytes());
    payload
}

pub(crate) fn encode_initiate_block_upload(index: u16, sub: u8, blksize: u8, crc: bool) -> [u8; 8] {
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
    payload
}

pub(crate) fn encode_start_block_upload() -> [u8; 8] {
    let mut payload = [0u8; 8];
    // Start upload CS: 10100011b
    payload[0] = 0xA3;
    payload
}

pub(crate) fn encode_block_upload_ack(ackseq: u8, blksize: u8) -> [u8; 8] {
    let mut payload = [0u8; 8];
    // Upload sub-block response CS: 10100010b
    payload[0] = 0xA2;
    payload[1] = ackseq;
    payload[2] = blksize;
    payload
}

pub(crate) fn encode_end_block_upload_confirmation() -> [u8; 8] {
    let mut payload = [0u8; 8];
    // Upload end response CS: 10100001b
    payload[0] = 0xA1;
    payload
}

#[cfg(test)]
mod tests {
    use super::*;

    const IDX: u16 = 0x2000;
    const SUB: u8 = 0x01;

    fn expedited_read() -> Pending {
        Pending::ExpeditedRead { index: IDX, sub: SUB }
    }

    /// Build a response payload with the multiplexer of the test request.
    fn response(cs: u8, tail: [u8; 4]) -> [u8; 8] {
        let idx = IDX.to_le_bytes();
        [cs, idx[0], idx[1], SUB, tail[0], tail[1], tail[2], tail[3]]
    }

    #[test]
    fn parse_rejects_short_frames() {
        let data = [0x43, 0x00, 0x20, 0x01, 0xAA];
        assert_eq!(
            parse_response(expedited_read(), &data).err(),
            Some(ParseError::InvalidResponse)
        );
    }

    #[test]
    fn parse_abort_frame() {
        // Abort 0x0602_0000: object does not exist
        let data = response(0x80, 0x0602_0000u32.to_le_bytes());
        assert_eq!(
            parse_response(expedited_read(), &data).err(),
            Some(ParseError::Abort(0x0602_0000))
        );
    }

    #[test]
    fn parse_expedited_upload_all_sizes() {
        // cs 0x4F/0x4B/0x47/0x43 = 1/2/3/4 valid bytes; bytes beyond len zeroed
        for (cs, expected_len, expected) in [
            (0x4Fu8, 1usize, [0xDD, 0, 0, 0]),
            (0x4B, 2, [0xDD, 0xCC, 0, 0]),
            (0x47, 3, [0xDD, 0xCC, 0xBB, 0]),
            (0x43, 4, [0xDD, 0xCC, 0xBB, 0xAA]),
        ] {
            let payload = response(cs, [0xDD, 0xCC, 0xBB, 0xAA]);
            match parse_response(expedited_read(), &payload).unwrap() {
                ParseOutcome::UploadInit(UploadInit::Expedited { data, len }) => {
                    assert_eq!(len, expected_len, "cs={cs:#04x}");
                    assert_eq!(data, expected, "cs={cs:#04x}");
                }
                _ => panic!("expected expedited UploadInit for cs={cs:#04x}"),
            }
        }
    }

    #[test]
    fn parse_expedited_upload_without_size_indication() {
        // cs 0x42: expedited, size not indicated -> all 4 bytes reported
        let payload = response(0x42, [0xDD, 0xCC, 0xBB, 0xAA]);
        assert_eq!(
            match parse_response(expedited_read(), &payload).unwrap() {
                ParseOutcome::UploadInit(init) => init,
                _ => panic!("expected UploadInit"),
            },
            UploadInit::Expedited { data: [0xDD, 0xCC, 0xBB, 0xAA], len: 4 }
        );
    }

    #[test]
    fn parse_segmented_upload_initiate() {
        // cs 0x41: segmented with size
        let payload = response(0x41, 300u32.to_le_bytes());
        assert_eq!(
            match parse_response(expedited_read(), &payload).unwrap() {
                ParseOutcome::UploadInit(init) => init,
                _ => panic!("expected UploadInit"),
            },
            UploadInit::Segmented { size: Some(300) }
        );

        // cs 0x40: segmented without size indication
        let payload = response(0x40, [0; 4]);
        assert_eq!(
            match parse_response(expedited_read(), &payload).unwrap() {
                ParseOutcome::UploadInit(init) => init,
                _ => panic!("expected UploadInit"),
            },
            UploadInit::Segmented { size: None }
        );
    }

    #[test]
    fn parse_expedited_upload_wrong_multiplexer() {
        let mut data = response(0x43, [0; 4]);
        data[3] = SUB + 1;
        assert_eq!(
            parse_response(expedited_read(), &data).err(),
            Some(ParseError::InvalidResponse)
        );
    }

    #[test]
    fn parse_download_response_ack() {
        let data = response(0x60, [0; 4]);
        let pending = Pending::ExpeditedWrite { index: IDX, sub: SUB };
        assert!(matches!(parse_response(pending, &data), Ok(ParseOutcome::Ack)));
    }

    #[test]
    fn parse_upload_segments_toggle_and_last() {
        // Middle segment, toggle 0, 7 bytes: cs = 0x00
        let data = [0x00, 1, 2, 3, 4, 5, 6, 7];
        let pending = Pending::UploadSegment { toggle: false };
        match parse_response(pending, &data).unwrap() {
            ParseOutcome::Segment(seg) => {
                assert!(!seg.last);
                assert_eq!(seg.len, 7);
                assert_eq!(seg.data, [1, 2, 3, 4, 5, 6, 7]);
            }
            _ => panic!("expected Segment"),
        }

        // Last segment, toggle 1, 3 valid bytes (n=4): cs = 000/1/100/1 = 0x19
        let data = [0x19, 1, 2, 3, 0, 0, 0, 0];
        let pending = Pending::UploadSegment { toggle: true };
        match parse_response(pending, &data).unwrap() {
            ParseOutcome::Segment(seg) => {
                assert!(seg.last);
                assert_eq!(seg.len, 3);
                assert_eq!(&seg.data[..3], &[1, 2, 3]);
            }
            _ => panic!("expected Segment"),
        }

        // Toggle mismatch is rejected
        let data = [0x10, 0, 0, 0, 0, 0, 0, 0];
        let pending = Pending::UploadSegment { toggle: false };
        assert_eq!(parse_response(pending, &data).err(), Some(ParseError::InvalidResponse));
    }

    #[test]
    fn parse_download_segment_response() {
        // cs = 001t0000: toggle must match
        let pending = Pending::DownloadSegment { toggle: true };
        assert!(matches!(
            parse_response(pending, &[0x30, 0, 0, 0, 0, 0, 0, 0]),
            Ok(ParseOutcome::Ack)
        ));
        assert_eq!(
            parse_response(pending, &[0x20, 0, 0, 0, 0, 0, 0, 0]).err(),
            Some(ParseError::InvalidResponse)
        );
    }

    #[test]
    fn parse_block_download_handshake() {
        // Initiate response with CRC support, blksize 127
        let data = response(0xA4, [127, 0, 0, 0]);
        let pending = Pending::BlockDownloadInitiate { index: IDX, sub: SUB };
        match parse_response(pending, &data).unwrap() {
            ParseOutcome::BlockInit(init) => {
                assert_eq!(init.blksize, 127);
                assert!(init.server_supports_crc);
            }
            _ => panic!("expected BlockInit"),
        }

        // Sub-block ack
        let data = [0xA2, 42, 64, 0, 0, 0, 0, 0];
        match parse_response(Pending::BlockDownloadAck, &data).unwrap() {
            ParseOutcome::BlockAck(ack) => {
                assert_eq!(ack.ackseq, 42);
                assert_eq!(ack.next_blksize, 64);
            }
            _ => panic!("expected BlockAck"),
        }

        // End response
        assert!(matches!(
            parse_response(Pending::BlockDownloadEnd, &[0xA1, 0, 0, 0, 0, 0, 0, 0]),
            Ok(ParseOutcome::Ack)
        ));
    }

    #[test]
    fn parse_block_upload_flow() {
        // Initiate response, size indicated (s bit), CRC supported (r bit): 0xC6
        let data = response(0xC6, 1000u32.to_le_bytes());
        let pending = Pending::BlockUploadInitiate { index: IDX, sub: SUB };
        match parse_response(pending, &data).unwrap() {
            ParseOutcome::BlockUploadInit(init) => {
                assert_eq!(init.size, Some(1000));
                assert!(init.server_supports_crc);
            }
            _ => panic!("expected BlockUploadInit"),
        }

        // Segment 3 of an ongoing transfer
        let data = [0x03, 9, 8, 7, 6, 5, 4, 3];
        match parse_response(Pending::BlockUploadActive, &data).unwrap() {
            ParseOutcome::BlockUploadSegment(seg) => {
                assert_eq!(seg.seqno, 3);
                assert!(!seg.last);
                assert_eq!(seg.data, [9, 8, 7, 6, 5, 4, 3]);
            }
            _ => panic!("expected BlockUploadSegment"),
        }

        // Last segment sets the c bit
        let data = [0x84, 0, 0, 0, 0, 0, 0, 0];
        match parse_response(Pending::BlockUploadActive, &data).unwrap() {
            ParseOutcome::BlockUploadSegment(seg) => {
                assert_eq!(seg.seqno, 4);
                assert!(seg.last);
            }
            _ => panic!("expected BlockUploadSegment"),
        }

        // Seqno 0 is invalid
        assert_eq!(
            parse_response(Pending::BlockUploadActive, &[0x00, 0, 0, 0, 0, 0, 0, 0]).err(),
            Some(ParseError::InvalidResponse)
        );

        // End frame carries the CRC: cs = 110nnn01
        let data = [0xC1, 0x34, 0x12, 0, 0, 0, 0, 0];
        match parse_response(Pending::BlockUploadEndWait, &data).unwrap() {
            ParseOutcome::BlockUploadEnd { crc } => assert_eq!(crc, 0x1234),
            _ => panic!("expected BlockUploadEnd"),
        }
    }

    #[test]
    fn encode_expedited_roundtrip_shapes() {
        assert_eq!(encode_upload_request(0x1018, 4), [0x40, 0x18, 0x10, 4, 0, 0, 0, 0]);
        // 2-byte expedited download: n=2 -> cs = 0b0010_1011
        assert_eq!(
            encode_download_expedited(0x2000, 1, &[0xAA, 0xBB]),
            [0x2B, 0x00, 0x20, 1, 0xAA, 0xBB, 0, 0]
        );
        assert_eq!(
            encode_initiate_segmented_download(0x2000, 1, 300),
            [0x21, 0x00, 0x20, 1, 44, 1, 0, 0]
        );
        // 3-byte last segment with toggle: n=4, cs = 000/1/100/1
        assert_eq!(
            encode_download_segment(&[1, 2, 3], true, true),
            [0x19, 1, 2, 3, 0, 0, 0, 0]
        );
        assert_eq!(encode_upload_segment_request(true)[0], 0x70);
    }

    #[test]
    fn encode_block_frames() {
        assert_eq!(
            encode_initiate_block_download(0x2000, 1, 1000, true),
            [0xC6, 0x00, 0x20, 1, 232, 3, 0, 0]
        );
        assert_eq!(encode_block_download_segment(&[5, 6], 3, true)[..3], [0x83, 5, 6]);
        // 2 unused bytes in last segment: cs = 110/010/01 = 0xC9
        assert_eq!(encode_end_block_download(2, 0x1234)[..3], [0xC9, 0x34, 0x12]);
        assert_eq!(
            encode_initiate_block_upload(0x2000, 1, 127, true),
            [0xA4, 0x00, 0x20, 1, 127, 0, 0, 0]
        );
        assert_eq!(encode_start_block_upload()[0], 0xA3);
        assert_eq!(encode_block_upload_ack(127, 64)[..3], [0xA2, 127, 64]);
        assert_eq!(encode_end_block_upload_confirmation()[0], 0xA1);
    }
}
