//! Virtual CPU context and execution state for the Micro-OS.

use std::fmt;

/// General-purpose register count in our virtual CPU.
pub const NUM_REGISTERS: usize = 8;

/// Virtual CPU registers and execution state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CpuContext {
    /// Program Counter (current instruction offset)
    pub pc: usize,
    /// Stack Pointer
    pub sp: usize,
    /// General Purpose Registers (r0..r7)
    pub regs: [u64; NUM_REGISTERS],
    /// Flags register (bit 0: Zero, bit 1: Carry, bit 2: Interrupt Enable)
    pub flags: u32,
    /// Number of CPU cycles consumed by this context
    pub cycles_executed: u64,
}

impl CpuContext {
    pub const FLAG_ZERO: u32 = 1 << 0;
    pub const FLAG_CARRY: u32 = 1 << 1;
    pub const FLAG_INTERRUPT_ENABLE: u32 = 1 << 2;

    pub fn new(entry_point: usize, stack_top: usize) -> Self {
        Self {
            pc: entry_point,
            sp: stack_top,
            regs: [0; NUM_REGISTERS],
            flags: Self::FLAG_INTERRUPT_ENABLE,
            cycles_executed: 0,
        }
    }

    pub fn reset(&mut self, entry_point: usize, stack_top: usize) {
        self.pc = entry_point;
        self.sp = stack_top;
        self.regs = [0; NUM_REGISTERS];
        self.flags = Self::FLAG_INTERRUPT_ENABLE;
        self.cycles_executed = 0;
    }
}

impl fmt::Display for CpuContext {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "PC: 0x{:04x} | SP: 0x{:04x} | FLAGS: 0x{:02x} | CYCLES: {}",
            self.pc, self.sp, self.flags, self.cycles_executed
        )
    }
}
