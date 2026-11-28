//! Dynamic Host Configuration Protocol (DHCP - RFC 2131).

use crate::net::ethernet::MacAddress;
use crate::net::ipv4::Ipv4Address;

pub const DHCP_CLIENT_PORT: u16 = 68;
pub const DHCP_SERVER_PORT: u16 = 67;
pub const DHCP_MAGIC_COOKIE: [u8; 4] = [99, 130, 83, 99]; // 0x63825363

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DhcpMessageType {
    Discover = 1,
    Offer    = 2,
    Request  = 3,
    Ack      = 5,
    Nak      = 6,
    Unknown(u8),
}

/// Network configuration acquired dynamically through DHCP.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NetworkConfig {
    pub ip_address: Ipv4Address,
    pub subnet_mask: Ipv4Address,
    pub gateway: Ipv4Address,
    pub dns_server: Ipv4Address,
    pub lease_time_secs: u32,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            ip_address: Ipv4Address::ZERO,
            subnet_mask: Ipv4Address::ZERO,
            gateway: Ipv4Address::ZERO,
            dns_server: Ipv4Address::ZERO,
            lease_time_secs: 0,
        }
    }
}

/// DHCP Packet Builder and Parser.
pub struct DhcpClient {
    mac: MacAddress,
    xid: u32,
}

impl DhcpClient {
    pub fn new(mac: MacAddress) -> Self {
        Self {
            mac,
            xid: 0x3903F326,
        }
    }

    /// Build a standard DHCP Discover broadcast packet.
    pub fn build_discover(&self) -> Vec<u8> {
        let mut p = vec![0u8; 240];
        // Op: BootRequest (1)
        p[0] = 1;
        // Hardware type: Ethernet (1)
        p[1] = 1;
        // Hardware address length: 6
        p[2] = 6;
        // Hops: 0
        p[3] = 0;
        // Transaction ID (xid)
        p[4] = (self.xid >> 24) as u8;
        p[5] = (self.xid >> 16) as u8;
        p[6] = (self.xid >> 8) as u8;
        p[7] = (self.xid & 0xFF) as u8;
        // Flags: Broadcast bit set (0x8000)
        p[10] = 0x80;
        p[11] = 0x00;
        // Client MAC address in chaddr field (offset 28..34)
        p[28..34].copy_from_slice(&self.mac.0);

        // Magic Cookie (offset 236..240)
        p[236..240].copy_from_slice(&DHCP_MAGIC_COOKIE);

        // Option 53: DHCP Message Type = Discover (1)
        p.extend_from_slice(&[53, 1, 1]);
        // Option 55: Parameter Request List (1=Subnet, 3=Router, 6=DNS)
        p.extend_from_slice(&[55, 3, 1, 3, 6]);
        // Option 255: End
        p.push(255);

        p
    }

    /// Build a DHCP Request packet to acknowledge the server's offer.
    pub fn build_request(&self, offered_ip: Ipv4Address, server_ip: Ipv4Address) -> Vec<u8> {
        let mut p = vec![0u8; 240];
        p[0] = 1;
        p[1] = 1;
        p[2] = 6;
        p[3] = 0;
        p[4] = (self.xid >> 24) as u8;
        p[5] = (self.xid >> 16) as u8;
        p[6] = (self.xid >> 8) as u8;
        p[7] = (self.xid & 0xFF) as u8;
        p[10] = 0x80;
        p[11] = 0x00;
        p[28..34].copy_from_slice(&self.mac.0);
        p[236..240].copy_from_slice(&DHCP_MAGIC_COOKIE);

        // Option 53: DHCP Message Type = Request (3)
        p.extend_from_slice(&[53, 1, 3]);
        // Option 50: Requested IP Address
        p.extend_from_slice(&[50, 4, offered_ip.0[0], offered_ip.0[1], offered_ip.0[2], offered_ip.0[3]]);
        // Option 54: Server Identifier
        p.extend_from_slice(&[54, 4, server_ip.0[0], server_ip.0[1], server_ip.0[2], server_ip.0[3]]);
        // Option 255: End
        p.push(255);

        p
    }

    /// Parse a DHCP Offer or Ack message and extract assigned IP configuration.
    pub fn parse_reply(&self, bytes: &[u8]) -> Result<(DhcpMessageType, NetworkConfig), &'static str> {
        if bytes.len() < 240 {
            return Err("DHCP message too short (< 240 bytes)");
        }
        if &bytes[236..240] != &DHCP_MAGIC_COOKIE {
            return Err("Invalid DHCP magic cookie");
        }

        let mut config = NetworkConfig::default();
        // yiaddr (Your IP Address) at offset 16..20
        config.ip_address = Ipv4Address([bytes[16], bytes[17], bytes[18], bytes[19]]);

        let mut msg_type = DhcpMessageType::Unknown(0);
        let mut i = 240;
        while i < bytes.len() {
            let opt = bytes[i];
            if opt == 255 {
                break; // End option
            }
            if opt == 0 {
                i += 1;
                continue;
            }
            if i + 1 >= bytes.len() {
                break;
            }
            let len = bytes[i + 1] as usize;
            i += 2;
            if i + len > bytes.len() {
                break;
            }

            match opt {
                53 if len >= 1 => {
                    msg_type = match bytes[i] {
                        1 => DhcpMessageType::Discover,
                        2 => DhcpMessageType::Offer,
                        3 => DhcpMessageType::Request,
                        5 => DhcpMessageType::Ack,
                        6 => DhcpMessageType::Nak,
                        x => DhcpMessageType::Unknown(x),
                    };
                }
                1 if len >= 4 => {
                    config.subnet_mask = Ipv4Address([bytes[i], bytes[i + 1], bytes[i + 2], bytes[i + 3]]);
                }
                3 if len >= 4 => {
                    config.gateway = Ipv4Address([bytes[i], bytes[i + 1], bytes[i + 2], bytes[i + 3]]);
                }
                6 if len >= 4 => {
                    config.dns_server = Ipv4Address([bytes[i], bytes[i + 1], bytes[i + 2], bytes[i + 3]]);
                }
                51 if len >= 4 => {
                    config.lease_time_secs = ((bytes[i] as u32) << 24)
                        | ((bytes[i + 1] as u32) << 16)
                        | ((bytes[i + 2] as u32) << 8)
                        | (bytes[i + 3] as u32);
                }
                _ => {}
            }
            i += len;
        }

        Ok((msg_type, config))
    }
}
