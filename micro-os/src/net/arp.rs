//! Address Resolution Protocol (ARP - RFC 826).

use crate::net::ethernet::MacAddress;
use crate::net::ipv4::Ipv4Address;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArpOperation {
    Request = 1,
    Reply   = 2,
    Unknown(u16),
}

impl From<u16> for ArpOperation {
    fn from(val: u16) -> Self {
        match val {
            1 => ArpOperation::Request,
            2 => ArpOperation::Reply,
            x => ArpOperation::Unknown(x),
        }
    }
}

/// Standard 28-byte ARP Packet for Ethernet and IPv4.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArpPacket {
    pub operation: ArpOperation,
    pub sender_mac: MacAddress,
    pub sender_ip: Ipv4Address,
    pub target_mac: MacAddress,
    pub target_ip: Ipv4Address,
}

impl ArpPacket {
    pub const PACKET_LEN: usize = 28;

    pub fn new_request(sender_mac: MacAddress, sender_ip: Ipv4Address, target_ip: Ipv4Address) -> Self {
        Self {
            operation: ArpOperation::Request,
            sender_mac,
            sender_ip,
            target_mac: MacAddress::ZERO,
            target_ip,
        }
    }

    pub fn new_reply(
        sender_mac: MacAddress,
        sender_ip: Ipv4Address,
        target_mac: MacAddress,
        target_ip: Ipv4Address,
    ) -> Self {
        Self {
            operation: ArpOperation::Reply,
            sender_mac,
            sender_ip,
            target_mac,
            target_ip,
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(Self::PACKET_LEN);
        // Hardware type: Ethernet (1)
        bytes.extend_from_slice(&[0x00, 0x01]);
        // Protocol type: IPv4 (0x0800)
        bytes.extend_from_slice(&[0x08, 0x00]);
        // Hardware size (6), Protocol size (4)
        bytes.push(6);
        bytes.push(4);
        // Opcode
        let op: u16 = match self.operation {
            ArpOperation::Request => 1,
            ArpOperation::Reply   => 2,
            ArpOperation::Unknown(x) => x,
        };
        bytes.push((op >> 8) as u8);
        bytes.push((op & 0xFF) as u8);
        // Sender MAC & IP
        bytes.extend_from_slice(&self.sender_mac.0);
        bytes.extend_from_slice(&self.sender_ip.0);
        // Target MAC & IP
        bytes.extend_from_slice(&self.target_mac.0);
        bytes.extend_from_slice(&self.target_ip.0);
        bytes
    }

    pub fn parse(bytes: &[u8]) -> Result<Self, &'static str> {
        if bytes.len() < Self::PACKET_LEN {
            return Err("ARP packet too short (< 28 bytes)");
        }
        let op = ((bytes[6] as u16) << 8) | (bytes[7] as u16);
        let mut s_mac = [0u8; 6];
        s_mac.copy_from_slice(&bytes[8..14]);
        let mut s_ip = [0u8; 4];
        s_ip.copy_from_slice(&bytes[14..18]);

        let mut t_mac = [0u8; 6];
        t_mac.copy_from_slice(&bytes[18..24]);
        let mut t_ip = [0u8; 4];
        t_ip.copy_from_slice(&bytes[24..28]);

        Ok(Self {
            operation: ArpOperation::from(op),
            sender_mac: MacAddress::new(s_mac),
            sender_ip: Ipv4Address(s_ip),
            target_mac: MacAddress::new(t_mac),
            target_ip: Ipv4Address(t_ip),
        })
    }
}

/// Dynamic ARP Cache mapping IPv4 addresses to physical MAC addresses.
#[derive(Debug, Default)]
pub struct ArpTable {
    entries: HashMap<Ipv4Address, MacAddress>,
}

impl ArpTable {
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }

    pub fn insert(&mut self, ip: Ipv4Address, mac: MacAddress) {
        self.entries.insert(ip, mac);
    }

    pub fn lookup(&self, ip: &Ipv4Address) -> Option<MacAddress> {
        self.entries.get(ip).copied()
    }
}
