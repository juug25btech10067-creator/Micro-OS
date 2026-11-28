//! Virtual Timer driver producing periodic tick events.

pub struct VirtualTimer {
    frequency_hz: u64,
    elapsed_ticks: usize,
}

impl VirtualTimer {
    pub fn new(frequency_hz: u64) -> Self {
        Self {
            frequency_hz,
            elapsed_ticks: 0,
        }
    }

    /// Advance time by one tick and return the new tick count.
    pub fn tick(&mut self) -> usize {
        self.elapsed_ticks += 1;
        self.elapsed_ticks
    }

    pub fn current_tick(&self) -> usize {
        self.elapsed_ticks
    }

    pub fn frequency(&self) -> u64 {
        self.frequency_hz
    }

    pub fn reset(&mut self) {
        self.elapsed_ticks = 0;
    }
}
