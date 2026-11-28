//! System Call Interface and User-Kernel Boundary Transition.

use crate::task::TaskId;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyscallId {
    Yield = 1,
    Sleep = 2,
    SpinlockAcquire = 3,
    SpinlockRelease = 4,
    MutexLock = 5,
    MutexUnlock = 6,
    Alloc = 7,
    Free = 8,
    Exit = 9,
}

#[derive(Debug, Clone)]
pub enum SyscallRequest {
    Yield,
    Sleep { ticks: usize },
    SpinlockAcquire { lock_id: usize },
    SpinlockRelease { lock_id: usize },
    MutexLock { mutex_id: usize },
    MutexUnlock { mutex_id: usize },
    Alloc { bytes: usize },
    Free { address: usize },
    Exit { code: i32 },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SyscallResult {
    Success,
    Allocated { address: usize },
    Blocked { reason: &'static str },
    Error(&'static str),
}

pub struct SyscallContext {
    pub caller: TaskId,
    pub request: SyscallRequest,
}
