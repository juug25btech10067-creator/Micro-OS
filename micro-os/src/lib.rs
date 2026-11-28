//! CA-6 Micro-OS Kernel Library
//!
//! Educational Micro-OS demonstrating core operating system mechanisms in Rust:
//! - Layer 1: Boot & System Init
//! - Layer 2: Diagnostics & Structured Logging
//! - Layer 3: Task Representation & Control Blocks
//! - Layer 4: Ready Queue
//! - Layer 5: Round Robin Scheduling
//! - Layer 6: Timer-Driven Scheduling & Preemption
//! - Layer 7: Synchronization (Specialization Track C: Spinlock, Ticket Lock, Mutex, Deadlock Prevention)
//! - Layer 8: Memory Allocation (Bump & Fixed-Size Block Free-List Allocator)
//! - Layer 9: System Interface & Syscalls
//! - Layer 10: Integrated Micro-OS Demonstration
//! - Internet & Networking Subsystem (Intel e1000, Ethernet, ARP, IPv4, ICMP, DHCP, HTTP)

pub mod kernel;
pub mod task;
pub mod scheduler;
pub mod sync;
pub mod memory;
pub mod drivers;
pub mod syscall;
pub mod net;

pub use kernel::{BootManager, BootState, SystemConfig, CpuContext, LogLevel, klog};
pub use task::{TaskId, TaskState, TaskControlBlock, TaskInstruction, VirtualStack};
pub use scheduler::{KernelEngine, ReadyQueue, SchedulerEvent};
pub use sync::{Spinlock, TicketSpinlock, SleepMutex, DeadlockManager, ContentionReport};
pub use memory::{BumpAllocator, BlockAllocator, MemoryStats};
pub use net::{NetworkStack, E1000Driver, EthernetFrame, Ipv4Packet, Ipv4Address, IcmpEchoPacket, DhcpClient, NetworkConfig, HttpRequest, HttpResponse};
