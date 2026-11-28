//! Spinlock implementations: Basic Test-and-Set and Fair Ticket Spinlock.

use crate::task::TaskId;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

/// Metrics collected per spinlock instance.
#[derive(Debug, Default, Clone)]
pub struct LockStats {
    pub total_acquisitions: usize,
    pub contended_acquisitions: usize,
    pub total_spin_cycles: u64,
    pub max_spin_cycles: u64,
    pub total_hold_cycles: u64,
}

impl LockStats {
    pub fn contention_rate(&self) -> f64 {
        if self.total_acquisitions == 0 {
            0.0
        } else {
            (self.contended_acquisitions as f64) / (self.total_acquisitions as f64) * 100.0
        }
    }

    pub fn avg_spin_cycles(&self) -> f64 {
        if self.contended_acquisitions == 0 {
            0.0
        } else {
            (self.total_spin_cycles as f64) / (self.contended_acquisitions as f64)
        }
    }
}

/// Basic Test-and-Set (TAS) Spinlock.
pub struct Spinlock {
    id: usize,
    name: &'static str,
    rank: u32,
    locked: AtomicBool,
    owner: AtomicUsize, // 0 = none, otherwise task_id.0 + 1
    stats: LockStats,
}

impl Spinlock {
    pub const fn new(id: usize, name: &'static str, rank: u32) -> Self {
        Self {
            id,
            name,
            rank,
            locked: AtomicBool::new(false),
            owner: AtomicUsize::new(0),
            stats: LockStats {
                total_acquisitions: 0,
                contended_acquisitions: 0,
                total_spin_cycles: 0,
                max_spin_cycles: 0,
                total_hold_cycles: 0,
            },
        }
    }

    pub fn id(&self) -> usize {
        self.id
    }

    pub fn name(&self) -> &'static str {
        self.name
    }

    pub fn rank(&self) -> u32 {
        self.rank
    }

    pub fn is_locked(&self) -> bool {
        self.locked.load(Ordering::Acquire)
    }

    pub fn owner(&self) -> Option<TaskId> {
        let val = self.owner.load(Ordering::Relaxed);
        if val == 0 {
            None
        } else {
            Some(TaskId(val - 1))
        }
    }

    /// Try to acquire the lock immediately without spinning.
    pub fn try_acquire(&mut self, task_id: TaskId) -> bool {
        if self.locked.compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed).is_ok() {
            self.owner.store(task_id.0 + 1, Ordering::Relaxed);
            self.stats.total_acquisitions += 1;
            true
        } else {
            false
        }
    }

    /// Attempt to acquire lock, simulating spin-wait cycles.
    /// Returns (success, spin_cycles_spent).
    pub fn acquire_with_spin(&mut self, task_id: TaskId, max_spins: u64) -> (bool, u64) {
        let mut spins: u64 = 0;
        let mut contended = false;

        while spins < max_spins {
            if self.locked.compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed).is_ok() {
                self.owner.store(task_id.0 + 1, Ordering::Relaxed);
                self.stats.total_acquisitions += 1;
                if contended {
                    self.stats.contended_acquisitions += 1;
                    self.stats.total_spin_cycles += spins;
                    if spins > self.stats.max_spin_cycles {
                        self.stats.max_spin_cycles = spins;
                    }
                }
                return (true, spins);
            }
            contended = true;
            spins += 1;
            // Simulated PAUSE/NOP instruction
            std::hint::spin_loop();
        }

        (false, spins)
    }

    /// Release the lock held by task_id.
    pub fn release(&mut self, task_id: TaskId) -> Result<(), &'static str> {
        if self.owner() != Some(task_id) {
            return Err("Spinlock release error: caller is not the lock owner");
        }
        self.owner.store(0, Ordering::Relaxed);
        self.locked.store(false, Ordering::Release);
        Ok(())
    }

    pub fn stats(&self) -> &LockStats {
        &self.stats
    }
}

/// Fair Ticket Spinlock ensuring FIFO grant order and starvation prevention.
pub struct TicketSpinlock {
    id: usize,
    name: &'static str,
    rank: u32,
    next_ticket: AtomicUsize,
    now_serving: AtomicUsize,
    owner: AtomicUsize,
    stats: LockStats,
}

impl TicketSpinlock {
    pub const fn new(id: usize, name: &'static str, rank: u32) -> Self {
        Self {
            id,
            name,
            rank,
            next_ticket: AtomicUsize::new(0),
            now_serving: AtomicUsize::new(0),
            owner: AtomicUsize::new(0),
            stats: LockStats {
                total_acquisitions: 0,
                contended_acquisitions: 0,
                total_spin_cycles: 0,
                max_spin_cycles: 0,
                total_hold_cycles: 0,
            },
        }
    }

    pub fn id(&self) -> usize {
        self.id
    }

    pub fn name(&self) -> &'static str {
        self.name
    }

    pub fn rank(&self) -> u32 {
        self.rank
    }

    /// Take a ticket to join the queue.
    pub fn take_ticket(&self) -> usize {
        self.next_ticket.fetch_add(1, Ordering::SeqCst)
    }

    /// Check if this ticket is currently being served.
    pub fn is_my_turn(&self, my_ticket: usize) -> bool {
        self.now_serving.load(Ordering::Acquire) == my_ticket
    }

    /// Grant ownership to task after ticket matches.
    pub fn claim(&mut self, task_id: TaskId, ticket: usize, spins: u64) {
        self.owner.store(task_id.0 + 1, Ordering::Relaxed);
        self.stats.total_acquisitions += 1;
        if spins > 0 {
            self.stats.contended_acquisitions += 1;
            self.stats.total_spin_cycles += spins;
            if spins > self.stats.max_spin_cycles {
                self.stats.max_spin_cycles = spins;
            }
        }
    }

    /// Release ticket lock by incrementing now_serving.
    pub fn release(&mut self, task_id: TaskId) -> Result<(), &'static str> {
        let val = self.owner.load(Ordering::Relaxed);
        if val != task_id.0 + 1 {
            return Err("TicketSpinlock release error: caller is not lock owner");
        }
        self.owner.store(0, Ordering::Relaxed);
        self.now_serving.fetch_add(1, Ordering::Release);
        Ok(())
    }

    pub fn stats(&self) -> &LockStats {
        &self.stats
    }
}
