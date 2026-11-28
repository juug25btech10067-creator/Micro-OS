//! IPv4 Network Layer Protocol.

use std::fmt;

/// 4-byte IPv4 Address.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Ipv4Address(pub [u8; 4]);

impl Ipv4Address {
    pub const BROADCAST: Self = Self([255, 255, 255, 255]);
    pub const LOOPBACK: Self  = Self([127, 0, 0, 1]);
    pub const ZERO: Self      = Self([0, 0, 0, 0]);

    pub const fn new(a: u8, b: u8, c: u8, d: u8) -> Self {
        Self([a, b, c, d])
    }

    pub fn to_u32(&self) -> u32 {
        ((self.0[0] as u32) << 24)
            | ((self.0[1] as u32) << 16)
            | ((self.0[2] as u32) << 8)
            | (self.0[3] as u32)
    }
}

impl fmt::Display for Ipv4Address {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}.{}", self.0[0], self.0[1], self.0[2], self.0[3])
    }
}

/// IPv4 Transport Protocol Codes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IpProtocol {
    Icmp = 1,
    Tcp  = 6,
    Udp  = 17,
    Unknown(u8),
}

impl From<u8> for IpProtocol {
    fn from(val: u8) -> Self {
        match val {
            1  => IpProtocol::Icmp,
            6  => IpProtocol::Tcp,
            17 => IpProtocol::Udp,
            x  => IpProtocol::Unknown(x),
        }
    }
}

impl From<IpProtocol> for u8 {
    fn from(proto: IpProtocol) -> Self {
        match proto {
            IpProtocol::Icmp => 1,
            IpProtocol::Tcp  => 6,
            IpProtocol::Udp  => 17,
            IpProtocol::Unknown(x) => x,
        }
    }
}

/// Standard 20-byte IPv4 Packet Header (without options).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ipv4Packet {
    pub src_ip: Ipv4Address,
    pub dest_ip: Ipv4Address,
    pub protocol: IpProtocol,
    pub ttl: u8,
    pub identification: u16,
    pub payload: Vec<u8>,
}

impl Ipv4Packet {
    pub const MIN_HEADER_LEN: usize = 20;

    pub fn new(src_ip: Ipv4Address, dest_ip: Ipv4Address, protocol: IpProtocol, payload: Vec<u8>) -> Self {
        Self {
            src_ip,
            dest_ip,
            protocol,
            ttl: 64,
            identification: 0x1234,
            payload,
        }
    }

    /// Calculate RFC 791 16-bit Internet Checksum.
    pub fn calculate_checksum(header: &[u8]) -> u16 {
        let mut sum: u32 = 0;
        let mut i = 0;
        while i < header.len() {
            // Skip the checksum field (offset 10..12) during computation
            if i == 10 {
                i += 2;
                continue;
            }
            let word = if i + 1 < header.len() {
                ((header[i] as u32) << 8) | (header[i + 1] as u32)
            } else {
                (header[i] as u32) << 8
            };
            sum += word;
            i += 2;
        }

        while (sum >> 16) != 0 {
            sum = (sum & 0xFFFF) + (sum >> 16);
        }

        !(sum as u16)
    }

    /// Serialize IPv4 Packet to wire bytes including calculated header checksum.
    pub fn to_bytes(&self) -> Vec<u8> {
        let total_length = (Self::MIN_HEADER_LEN + self.payload.len()) as u16;
        let mut bytes = Vec::with_capacity(total_length as usize);

        // Version (4) & IHL (5 words = 20 bytes)
        bytes.push(0x45);
        // Type of Service
        bytes.push(0x00);
        // Total Length
        bytes.push((total_length >> 8) as u8);
        bytes.push((total_length & 0xFF) as u8);
        // Identification
        bytes.push((self.identification >> 8) as u8);
        bytes.push((self.identification & 0xFF) as u8);
        // Flags & Fragment Offset (Don't Fragment = 0x4000)
        bytes.push(0x40);
        bytes.push(0x00);
        // TTL
        bytes.push(self.ttl);
        // Protocol
        bytes.push(self.protocol.into());
        // Checksum placeholder (0x0000)
        bytes.push(0x00);
        bytes.push(0x00);
        // Source IP
        bytes.extend_from_slice(&self.src_ip.0);
        // Destination IP
        bytes.extend_from_slice(&self.dest_ip.0);

        // Compute checksum over 20-byte header
        let csum = Self::calculate_checksum(&bytes[0..20]);
        bytes[10] = (csum >> 8) as u8;
        bytes[11] = (csum & 0xFF) as u8;

        // Append payload
        bytes.extend_from_slice(&self.payload);
        bytes
    }

    /// Parse an IPv4 packet from raw bytes.
    pub fn parse(bytes: &[u8]) -> Result<Self, &'static str> {
        if bytes.len() < Self::MIN_HEADER_LEN {
            return Err("IPv4 packet too short (< 20 bytes)");
        }

        let ver_ihl = bytes[0];
        let version = ver_ihl >> 4;
        let ihl = (ver_ihl & 0x0F) as usize * 4;

        if version != 4 {
            return Err("Invalid IP version (not IPv4)");
        }
        if ihl < Self::MIN_HEADER_LEN || bytes.len() < ihl {
            return Err("Invalid IHL header length");
        }

        let total_len = (((bytes[2] as usize) << 8) | (bytes[3] as usize)).min(bytes.len());
        let id = ((bytes[4] as u16) << 8) | (bytes[5] as u16);
        let ttl = bytes[8];
        let protocol = IpProtocol::from(bytes[9]);

        let mut src = [0u8; 4];
        src.copy_from_slice(&bytes[12..16]);

        let mut dest = [0u8; 4];
        dest.copy_from_slice(&bytes[16..20]);

        let payload = if total_len > ihl {
            bytes[ihl..total_len].to_vec()
        } else {
            Vec::new()
        };

        Ok(Self {
            src_ip: Ipv4Address(src),
            dest_ip: Ipv4Address(dest),
            protocol,
            ttl,
            identification: id,
            payload,
        })
    }
}
