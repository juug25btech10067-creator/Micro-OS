//! User Datagram Protocol (UDP - RFC 768).

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UdpPacket {
    pub src_port: u16,
    pub dest_port: u16,
    pub payload: Vec<u8>,
}

impl UdpPacket {
    pub const HEADER_LEN: usize = 8;

    pub fn new(src_port: u16, dest_port: u16, payload: Vec<u8>) -> Self {
        Self {
            src_port,
            dest_port,
            payload,
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let len = (Self::HEADER_LEN + self.payload.len()) as u16;
        let mut bytes = Vec::with_capacity(len as usize);
        bytes.push((self.src_port >> 8) as u8);
        bytes.push((self.src_port & 0xFF) as u8);
        bytes.push((self.dest_port >> 8) as u8);
        bytes.push((self.dest_port & 0xFF) as u8);
        bytes.push((len >> 8) as u8);
        bytes.push((len & 0xFF) as u8);
        // Checksum (optional in IPv4, set 0x0000)
        bytes.push(0x00);
        bytes.push(0x00);
        bytes.extend_from_slice(&self.payload);
        bytes
    }

    pub fn parse(bytes: &[u8]) -> Result<Self, &'static str> {
        if bytes.len() < Self::HEADER_LEN {
            return Err("UDP packet too short (< 8 bytes)");
        }
        let src = ((bytes[0] as u16) << 8) | (bytes[1] as u16);
        let dest = ((bytes[2] as u16) << 8) | (bytes[3] as u16);
        let len = (((bytes[4] as usize) << 8) | (bytes[5] as usize)).min(bytes.len());
        let payload = if len > Self::HEADER_LEN {
            bytes[Self::HEADER_LEN..len].to_vec()
        } else {
            Vec::new()
        };

        Ok(Self {
            src_port: src,
            dest_port: dest,
            payload,
        })
    }
}
