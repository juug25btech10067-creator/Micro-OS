//! Deadlock Prevention via Hierarchical Resource Ordering (Lock Ranking).

use crate::task::TaskId;
use std::collections::HashMap;

/// Error returned when a lock acquisition violates the strict hierarchical ranking rule.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeadlockRiskError {
    pub task_id: TaskId,
    pub requested_lock_id: usize,
    pub requested_rank: u32,
    pub max_held_rank: u32,
}

impl std::fmt::Display for DeadlockRiskError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Deadlock Invariant Violation: {} holds lock rank {} and requested lower/equal rank {} (Lock-{})",
            self.task_id, self.max_held_rank, self.requested_rank, self.requested_lock_id
        )
    }
}

/// Deadlock Prevention Manager enforcing strict hierarchical lock acquisition.
#[derive(Debug, Default)]
pub struct DeadlockManager {
    /// Tracks list of currently held lock ranks per task
    task_held_ranks: HashMap<TaskId, Vec<(usize, u32)>>, // Vec of (lock_id, rank)
    total_violations_prevented: usize,
}

impl DeadlockManager {
    pub fn new() -> Self {
        Self {
            task_held_ranks: HashMap::new(),
            total_violations_prevented: 0,
        }
    }

    /// Check if acquiring lock_id with rank is legal under hierarchical ordering.
    pub fn validate_acquisition(
        &mut self,
        task_id: TaskId,
        lock_id: usize,
        rank: u32,
    ) -> Result<(), DeadlockRiskError> {
        if let Some(held) = self.task_held_ranks.get(&task_id) {
            if let Some(&(_, max_rank)) = held.iter().max_by_key(|(_, r)| *r) {
                if rank <= max_rank {
                    self.total_violations_prevented += 1;
                    return Err(DeadlockRiskError {
                        task_id,
                        requested_lock_id: lock_id,
                        requested_rank: rank,
                        max_held_rank: max_rank,
                    });
                }
            }
        }
        Ok(())
    }

    /// Record successful acquisition of a lock by task.
    pub fn record_acquired(&mut self, task_id: TaskId, lock_id: usize, rank: u32) {
        self.task_held_ranks
            .entry(task_id)
            .or_default()
            .push((lock_id, rank));
    }

    /// Record release of a lock by task.
    pub fn record_released(&mut self, task_id: TaskId, lock_id: usize) {
        if let Some(held) = self.task_held_ranks.get_mut(&task_id) {
            if let Some(pos) = held.iter().position(|(id, _)| *id == lock_id) {
                held.remove(pos);
            }
        }
    }

    pub fn violations_count(&self) -> usize {
        self.total_violations_prevented
    }
}
