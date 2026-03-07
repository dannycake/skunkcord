// Copyright (c) Skunk Ventures LLC
// Last modified: 2025-03-07
// SPDX-License-Identifier: MIT

//! Voice UDP audio connection
//!
//! Handles the UDP connection for sending/receiving encrypted audio data.
//! Implements IP discovery, RTP packet construction, and audio encryption.

/// RTP header (12 bytes)
/// ```text
/// 0                   1                   2                   3
/// 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
/// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
/// |V=2|P|X|  CC   |M|     PT      |       sequence number         |
/// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
/// |                           timestamp                           |
/// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
/// |                             SSRC                              |
/// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
/// ```
#[derive(Debug, Clone)]
pub struct RtpHeader {
    /// RTP version (always 2)
    pub version: u8,
    /// Padding flag
    pub padding: bool,
    /// Extension flag
    pub extension: bool,
    /// CSRC count
    pub csrc_count: u8,
    /// Marker bit
    pub marker: bool,
    /// Payload type (0x78 = 120 for Discord Opus)
    pub payload_type: u8,
    /// Sequence number (wraps at u16::MAX)
    pub sequence: u16,
    /// Timestamp (increments by frame size, e.g., 960 for 20ms at 48kHz)
    pub timestamp: u32,
    /// Synchronization source identifier
    pub ssrc: u32,
}

impl RtpHeader {
    /// Create a new RTP header for Discord voice
    pub fn new(ssrc: u32) -> Self {
        Self {
            version: 2,
            padding: false,
            extension: false,
            csrc_count: 0,
            marker: false,
            payload_type: 0x78, // 120 = Opus
            sequence: 0,
            timestamp: 0,
            ssrc,
        }
    }

    /// Serialize the header to bytes (12 bytes)
    pub fn to_bytes(&self) -> [u8; 12] {
        let mut buf = [0u8; 12];

        // Byte 0: V(2) P(1) X(1) CC(4)
        buf[0] = (self.version << 6)
            | ((self.padding as u8) << 5)
            | ((self.extension as u8) << 4)
            | (self.csrc_count & 0x0F);

        // Byte 1: M(1) PT(7)
        buf[1] = ((self.marker as u8) << 7) | (self.payload_type & 0x7F);

        // Bytes 2-3: Sequence number (big-endian)
        buf[2..4].copy_from_slice(&self.sequence.to_be_bytes());

        // Bytes 4-7: Timestamp (big-endian)
        buf[4..8].copy_from_slice(&self.timestamp.to_be_bytes());

        // Bytes 8-11: SSRC (big-endian)
        buf[8..12].copy_from_slice(&self.ssrc.to_be_bytes());

        buf
    }

    /// Parse an RTP header from bytes
    pub fn from_bytes(data: &[u8]) -> Option<Self> {
        if data.len() < 12 {
            return None;
        }

        Some(Self {
            version: (data[0] >> 6) & 0x03,
            padding: (data[0] >> 5) & 0x01 == 1,
            extension: (data[0] >> 4) & 0x01 == 1,
            csrc_count: data[0] & 0x0F,
            marker: (data[1] >> 7) & 0x01 == 1,
            payload_type: data[1] & 0x7F,
            sequence: u16::from_be_bytes([data[2], data[3]]),
            timestamp: u32::from_be_bytes([data[4], data[5], data[6], data[7]]),
            ssrc: u32::from_be_bytes([data[8], data[9], data[10], data[11]]),
        })
    }

    /// Advance to next packet (increment sequence and timestamp)
    pub fn advance(&mut self, frame_size: u32) {
        self.sequence = self.sequence.wrapping_add(1);
        self.timestamp = self.timestamp.wrapping_add(frame_size);
    }
}

/// IP Discovery packet (for finding our external IP/port)
///
/// Sent to the voice UDP server to discover our external IP address
/// and port, which is needed for the Select Protocol step.
pub fn build_ip_discovery_packet(ssrc: u32) -> Vec<u8> {
    let mut packet = vec![0u8; 74];
    // Type: 0x0001 (request)
    packet[0] = 0x00;
    packet[1] = 0x01;
    // Length: 70
    packet[2] = 0x00;
    packet[3] = 0x46;
    // SSRC
    packet[4..8].copy_from_slice(&ssrc.to_be_bytes());
    // Address and port fields are zeroed (server fills them in response)
    packet
}

/// Parse IP Discovery response to get our external IP and port
pub fn parse_ip_discovery_response(data: &[u8]) -> Option<(String, u16)> {
    if data.len() < 74 {
        return None;
    }

    // Response type should be 0x0002
    if data[0] != 0x00 || data[1] != 0x02 {
        return None;
    }

    // IP address starts at byte 8, null-terminated string
    let ip_bytes = &data[8..72];
    let ip_end = ip_bytes
        .iter()
        .position(|&b| b == 0)
        .unwrap_or(ip_bytes.len());
    let ip = String::from_utf8_lossy(&ip_bytes[..ip_end]).to_string();

    // Port is last 2 bytes (big-endian)
    let port = u16::from_be_bytes([data[72], data[73]]);

    Some((ip, port))
}

/// Opus audio constants
pub mod opus {
    /// Sample rate (48 kHz)
    pub const SAMPLE_RATE: u32 = 48000;
    /// Channels (stereo)
    pub const CHANNELS: u8 = 2;
    /// Frame duration in milliseconds
    pub const FRAME_DURATION_MS: u32 = 20;
    /// Samples per frame (48000 * 20 / 1000)
    pub const FRAME_SIZE: u32 = SAMPLE_RATE * FRAME_DURATION_MS / 1000; // 960
    /// Silence frame — 5 bytes of Opus silence (stereo)
    pub const SILENCE_FRAME: &[u8] = &[0xF8, 0xFF, 0xFE, 0x00, 0x00];
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rtp_header_roundtrip() {
        let header = RtpHeader {
            version: 2,
            padding: false,
            extension: false,
            csrc_count: 0,
            marker: false,
            payload_type: 0x78,
            sequence: 1234,
            timestamp: 567890,
            ssrc: 0xDEADBEEF,
        };

        let bytes = header.to_bytes();
        assert_eq!(bytes.len(), 12);

        let parsed = RtpHeader::from_bytes(&bytes).unwrap();
        assert_eq!(parsed.version, 2);
        assert_eq!(parsed.payload_type, 0x78);
        assert_eq!(parsed.sequence, 1234);
        assert_eq!(parsed.timestamp, 567890);
        assert_eq!(parsed.ssrc, 0xDEADBEEF);
    }

    #[test]
    fn test_rtp_advance() {
        let mut header = RtpHeader::new(12345);
        assert_eq!(header.sequence, 0);
        assert_eq!(header.timestamp, 0);

        header.advance(960);
        assert_eq!(header.sequence, 1);
        assert_eq!(header.timestamp, 960);

        header.advance(960);
        assert_eq!(header.sequence, 2);
        assert_eq!(header.timestamp, 1920);
    }

    #[test]
    fn test_rtp_sequence_wraps() {
        let mut header = RtpHeader::new(1);
        header.sequence = u16::MAX;
        header.advance(960);
        assert_eq!(header.sequence, 0); // Should wrap
    }

    #[test]
    fn test_ip_discovery_packet() {
        let packet = build_ip_discovery_packet(0x12345678);
        assert_eq!(packet.len(), 74);
        assert_eq!(packet[0..2], [0x00, 0x01]); // Request type
        assert_eq!(packet[4..8], 0x12345678u32.to_be_bytes()); // SSRC
    }

    #[test]
    fn test_parse_ip_discovery_response() {
        let mut response = vec![0u8; 74];
        response[0] = 0x00;
        response[1] = 0x02; // Response type
                            // IP at bytes 8-71 (null-terminated)
        let ip = b"1.2.3.4";
        response[8..8 + ip.len()].copy_from_slice(ip);
        // Port at bytes 72-73 (big-endian)
        response[72] = 0x1F; // port 8080
        response[73] = 0x90;

        let (parsed_ip, parsed_port) = parse_ip_discovery_response(&response).unwrap();
        assert_eq!(parsed_ip, "1.2.3.4");
        assert_eq!(parsed_port, 8080);
    }

    #[test]
    fn test_opus_constants() {
        assert_eq!(opus::SAMPLE_RATE, 48000);
        assert_eq!(opus::FRAME_SIZE, 960);
        assert_eq!(opus::SILENCE_FRAME.len(), 5);
    }

    #[test]
    fn test_rtp_header_too_short() {
        assert!(RtpHeader::from_bytes(&[0u8; 11]).is_none());
        assert!(RtpHeader::from_bytes(&[]).is_none());
    }
}
