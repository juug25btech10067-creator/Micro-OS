//! Kernel Boot Layer and System Initialization.

use crate::kernel::cpu::CpuContext;
use crate::kernel::logger::{kinfo, LogLevel};

pub const KERNEL_NAME: &str = "Micro-OS (Rust)";
pub const KERNEL_VERSION: &str = "0.1.0-ca6-concurrency";
pub const KERNEL_TARGET: &str = "host-emulated-rv64/x86_64";

/// System configuration parameters established during boot.
#[derive(Debug, Clone)]
pub struct SystemConfig {
    pub quantum_ticks: usize,
    pub heap_size_bytes: usize,
    pub max_tasks: usize,
    pub timer_frequency_hz: u64,
}

impl Default for SystemConfig {
    fn default() -> Self {
        Self {
            quantum_ticks: 2,
            heap_size_bytes: 64 * 1024, // 64 KB educational heap
            max_tasks: 32,
            timer_frequency_hz: 100,
        }
    }
}

/// Boot state machine tracking initialization stages.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BootState {
    PowerOn,
    InitializingConsole,
    InitializingMemory,
    InitializingInterrupts,
    InitializingScheduler,
    KernelOperational,
}

pub struct BootManager {
    state: BootState,
    config: SystemConfig,
    boot_context: CpuContext,
}

impl BootManager {
    pub fn new(config: SystemConfig) -> Self {
        Self {
            state: BootState::PowerOn,
            config,
            boot_context: CpuContext::new(0x8000_0000, 0x8001_0000),
        }
    }

    pub fn state(&self) -> BootState {
        self.state
    }

    pub fn config(&self) -> &SystemConfig {
        &self.config
    }

    pub fn boot(&mut self) -> Result<(), &'static str> {
        self.state = BootState::InitializingConsole;
        self.print_banner();
        kinfo!("BOOT", "Layer 1: Kernel console established");

        self.state = BootState::InitializingMemory;
        kinfo!(
            "BOOT",
            "Layer 8: Initializing educational kernel memory ({} KB)",
            self.config.heap_size_bytes / 1024
        );

        self.state = BootState::InitializingInterrupts;
        kinfo!(
            "BOOT",
            "Layer 6: Setting up virtual timer interrupt ({} Hz)",
            self.config.timer_frequency_hz
        );

        self.state = BootState::InitializingScheduler;
        kinfo!(
            "BOOT",
            "Layer 5: Round Robin scheduler initialized (Quantum = {} ticks)",
            self.config.quantum_ticks
        );

        self.state = BootState::KernelOperational;
        kinfo!("BOOT", "Layer 10: Kernel initialization complete. Transitioning to operational mode.");
        Ok(())
    }

    fn print_banner(&self) {
        println!("============================================================");
        println!("       {} - v{}", KERNEL_NAME, KERNEL_VERSION);
        println!("       Track C: Concurrency Engineering Edition             ");
        println!("       Target: {}", KERNEL_TARGET);
        println!("============================================================");
    }
}
