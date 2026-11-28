//! Internet Control Message Protocol (ICMP - RFC 792: Ping / Echo).

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IcmpType {
    EchoReply   = 0,
    EchoRequest = 8,
    Unknown(u8),
}

impl From<u8> for IcmpType {
    fn from(val: u8) -> Self {
        match val {
            0 => IcmpType::EchoReply,
            8 => IcmpType::EchoRequest,
            x => IcmpType::Unknown(x),
        }
    }
}

/// ICMP Echo Message.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IcmpEchoPacket {
    pub icmp_type: IcmpType,
    pub code: u8,
    pub identifier: u16,
    pub sequence_number: u16,
    pub payload: Vec<u8>,
}

impl IcmpEchoPacket {
    pub const HEADER_LEN: usize = 8;

    pub fn new_request(identifier: u16, sequence_number: u16, payload: Vec<u8>) -> Self {
        Self {
            icmp_type: IcmpType::EchoRequest,
            code: 0,
            identifier,
            sequence_number,
            payload,
        }
    }

    pub fn new_reply(identifier: u16, sequence_number: u16, payload: Vec<u8>) -> Self {
        Self {
            icmp_type: IcmpType::EchoReply,
            code: 0,
            identifier,
            sequence_number,
            payload,
        }
    }

    pub fn calculate_checksum(bytes: &[u8]) -> u16 {
        let mut sum: u32 = 0;
        let mut i = 0;
        while i < bytes.len() {
            if i == 2 {
                i += 2;
                continue;
            }
            let word = if i + 1 < bytes.len() {
                ((bytes[i] as u32) << 8) | (bytes[i + 1] as u32)
            } else {
                (bytes[i] as u32) << 8
            };
            sum += word;
            i += 2;
        }

        while (sum >> 16) != 0 {
            sum = (sum & 0xFFFF) + (sum >> 16);
        }

        !(sum as u16)
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(Self::HEADER_LEN + self.payload.len());
        let type_code = match self.icmp_type {
            IcmpType::EchoReply => 0,
            IcmpType::EchoRequest => 8,
            IcmpType::Unknown(x) => x,
        };
        bytes.push(type_code);
        bytes.push(self.code);
        bytes.push(0x00); // Checksum placeholder
        bytes.push(0x00);
        bytes.push((self.identifier >> 8) as u8);
        bytes.push((self.identifier & 0xFF) as u8);
        bytes.push((self.sequence_number >> 8) as u8);
        bytes.push((self.sequence_number & 0xFF) as u8);
        bytes.extend_from_slice(&self.payload);

        let csum = Self::calculate_checksum(&bytes);
        bytes[2] = (csum >> 8) as u8;
        bytes[3] = (csum & 0xFF) as u8;

        bytes
    }

    pub fn parse(bytes: &[u8]) -> Result<Self, &'static str> {
        if bytes.len() < Self::HEADER_LEN {
            return Err("ICMP packet too short (< 8 bytes)");
        }
        let icmp_type = IcmpType::from(bytes[0]);
        let code = bytes[1];
        let id = ((bytes[4] as u16) << 8) | (bytes[5] as u16);
        let seq = ((bytes[6] as u16) << 8) | (bytes[7] as u16);
        let payload = bytes[Self::HEADER_LEN..].to_vec();

        Ok(Self {
            icmp_type,
            code,
            identifier: id,
            sequence_number: seq,
            payload,
        })
    }
}
