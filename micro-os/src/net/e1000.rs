//! Intel 82540EM (e1000) Gigabit Ethernet Controller Driver.

use crate::net::ethernet::{EthernetFrame, MacAddress};
use std::collections::VecDeque;

pub const E1000_VENDOR_ID: u16 = 0x8086;
pub const E1000_DEVICE_ID: u16 = 0x100E;

/// Hardware Statistics for the e1000 NIC.
#[derive(Debug, Default, Clone)]
pub struct NicStats {
    pub packets_transmitted: usize,
    pub packets_received: usize,
    pub bytes_transmitted: usize,
    pub bytes_received: usize,
    pub tx_errors: usize,
    pub rx_errors: usize,
}

/// Intel 82540EM Network Interface Card Driver.
pub struct E1000Driver {
    pub mac_address: MacAddress,
    pub link_up: bool,
    pub tx_ring: VecDeque<Vec<u8>>,
    pub rx_ring: VecDeque<Vec<u8>>,
    pub stats: NicStats,
}

impl E1000Driver {
    pub fn new(mac: MacAddress) -> Self {
        Self {
            mac_address: mac,
            link_up: true,
            tx_ring: VecDeque::new(),
            rx_ring: VecDeque::new(),
            stats: NicStats::default(),
        }
    }

    /// Transmit a raw Ethernet packet via the e1000 TX ring.
    pub fn transmit(&mut self, packet: &[u8]) -> Result<(), &'static str> {
        if !self.link_up {
            self.stats.tx_errors += 1;
            return Err("Link Down: Cannot transmit packet");
        }
        self.stats.packets_transmitted += 1;
        self.stats.bytes_transmitted += packet.len();
        self.tx_ring.push_back(packet.to_vec());
        Ok(())
    }

    /// Transmit a structured Ethernet frame.
    pub fn transmit_frame(&mut self, frame: &EthernetFrame) -> Result<(), &'static str> {
        let bytes = frame.to_bytes();
        self.transmit(&bytes)
    }

    /// Receive the next incoming packet from the e1000 RX ring.
    pub fn receive(&mut self) -> Option<Vec<u8>> {
        if let Some(packet) = self.rx_ring.pop_front() {
            self.stats.packets_received += 1;
            self.stats.bytes_received += packet.len();
            Some(packet)
        } else {
            None
        }
    }

    /// Push an incoming packet to the RX ring (simulating hardware DMA transfer).
    pub fn inject_rx(&mut self, packet: Vec<u8>) {
        self.rx_ring.push_back(packet);
    }
}
