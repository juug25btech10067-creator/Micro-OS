//! Sleeping Mutex implementation integrated with kernel task blocking.

use crate::task::TaskId;
use std::collections::VecDeque;

/// Statistics for Sleeping Mutex contention and sleep overhead.
#[derive(Debug, Default, Clone)]
pub struct MutexStats {
    pub total_acquisitions: usize,
    pub contested_acquisitions: usize,
    pub tasks_blocked_count: usize,
    pub total_blocked_ticks: usize,
    pub max_blocked_ticks: usize,
}

impl MutexStats {
    pub fn contention_rate(&self) -> f64 {
        if self.total_acquisitions == 0 {
            0.0
        } else {
            (self.contested_acquisitions as f64) / (self.total_acquisitions as f64) * 100.0
        }
    }
}

/// Sleeping Mutex that blocks tasks instead of burning CPU cycles in spinloops.
pub struct SleepMutex {
    id: usize,
    name: &'static str,
    rank: u32,
    owner: Option<TaskId>,
    wait_queue: VecDeque<(TaskId, usize)>, // (task_id, block_start_tick)
    stats: MutexStats,
}

impl SleepMutex {
    pub fn new(id: usize, name: &'static str, rank: u32) -> Self {
        Self {
            id,
            name,
            rank,
            owner: None,
            wait_queue: VecDeque::new(),
            stats: MutexStats::default(),
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

    pub fn owner(&self) -> Option<TaskId> {
        self.owner
    }

    pub fn is_locked(&self) -> bool {
        self.owner.is_some()
    }

    pub fn wait_queue_len(&self) -> usize {
        self.wait_queue.len()
    }

    /// Try to acquire without blocking. Returns true if acquired.
    pub fn try_lock(&mut self, task_id: TaskId) -> bool {
        if self.owner.is_none() {
            self.owner = Some(task_id);
            self.stats.total_acquisitions += 1;
            true
        } else {
            false
        }
    }

    /// Block a task on this mutex when it cannot acquire the lock.
    pub fn block_task(&mut self, task_id: TaskId, current_tick: usize) {
        self.wait_queue.push_back((task_id, current_tick));
        self.stats.contested_acquisitions += 1;
        self.stats.tasks_blocked_count += 1;
    }

    /// Release mutex ownership and wake up the next task from the wait queue.
    /// Returns Some((woken_task_id, blocked_duration_ticks)) if a task was waiting.
    pub fn unlock(&mut self, caller: TaskId, current_tick: usize) -> Result<Option<(TaskId, usize)>, &'static str> {
        if self.owner != Some(caller) {
            return Err("Mutex unlock error: caller is not current lock owner");
        }

        if let Some((next_task, block_tick)) = self.wait_queue.pop_front() {
            let blocked_ticks = current_tick.saturating_sub(block_tick);
            self.stats.total_blocked_ticks += blocked_ticks;
            if blocked_ticks > self.stats.max_blocked_ticks {
                self.stats.max_blocked_ticks = blocked_ticks;
            }

            // Direct handoff of ownership to the unblocked task
            self.owner = Some(next_task);
            self.stats.total_acquisitions += 1;
            Ok(Some((next_task, blocked_ticks)))
        } else {
            self.owner = None;
            Ok(None)
        }
    }

    pub fn stats(&self) -> &MutexStats {
        &self.stats
    }
}
