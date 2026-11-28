//! Ethernet II Data-Link Framing Layer.

use std::fmt;

/// 6-byte IEEE 802.3 MAC Address.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MacAddress(pub [u8; 6]);

impl MacAddress {
    pub const BROADCAST: Self = Self([0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF]);
    pub const ZERO: Self = Self([0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);

    pub fn new(b: [u8; 6]) -> Self {
        Self(b)
    }

    pub fn is_broadcast(&self) -> bool {
        *self == Self::BROADCAST
    }
}

impl fmt::Display for MacAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}",
            self.0[0], self.0[1], self.0[2], self.0[3], self.0[4], self.0[5]
        )
    }
}

/// Standard Ethernet Protocol Identifiers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EtherType {
    Ipv4 = 0x0800,
    Arp  = 0x0806,
    Ipv6 = 0x86DD,
    Unknown(u16),
}

impl From<u16> for EtherType {
    fn from(val: u16) -> Self {
        match val {
            0x0800 => EtherType::Ipv4,
            0x0806 => EtherType::Arp,
            0x86DD => EtherType::Ipv6,
            other => EtherType::Unknown(other),
        }
    }
}

impl From<EtherType> for u16 {
    fn from(eth: EtherType) -> Self {
        match eth {
            EtherType::Ipv4 => 0x0800,
            EtherType::Arp  => 0x0806,
            EtherType::Ipv6 => 0x86DD,
            EtherType::Unknown(val) => val,
        }
    }
}

/// An Ethernet II Frame with 14-byte header and payload.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EthernetFrame {
    pub dest_mac: MacAddress,
    pub src_mac: MacAddress,
    pub ethertype: EtherType,
    pub payload: Vec<u8>,
}

impl EthernetFrame {
    pub const HEADER_LEN: usize = 14;

    pub fn new(dest_mac: MacAddress, src_mac: MacAddress, ethertype: EtherType, payload: Vec<u8>) -> Self {
        Self {
            dest_mac,
            src_mac,
            ethertype,
            payload,
        }
    }

    /// Serialize Ethernet Frame to wire format.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(Self::HEADER_LEN + self.payload.len());
        bytes.extend_from_slice(&self.dest_mac.0);
        bytes.extend_from_slice(&self.src_mac.0);
        let proto: u16 = self.ethertype.into();
        bytes.push((proto >> 8) as u8);
        bytes.push((proto & 0xFF) as u8);
        bytes.extend_from_slice(&self.payload);
        bytes
    }

    /// Parse an Ethernet Frame from raw wire bytes.
    pub fn parse(bytes: &[u8]) -> Result<Self, &'static str> {
        if bytes.len() < Self::HEADER_LEN {
            return Err("Ethernet frame too short (< 14 bytes)");
        }

        let mut dest = [0u8; 6];
        dest.copy_from_slice(&bytes[0..6]);

        let mut src = [0u8; 6];
        src.copy_from_slice(&bytes[6..12]);

        let proto = ((bytes[12] as u16) << 8) | (bytes[13] as u16);
        let payload = bytes[Self::HEADER_LEN..].to_vec();

        Ok(Self {
            dest_mac: MacAddress::new(dest),
            src_mac: MacAddress::new(src),
            ethertype: EtherType::from(proto),
            payload,
        })
    }
}
