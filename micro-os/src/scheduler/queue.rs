//! Ready Queue kernel data structure.

use crate::task::TaskId;
use std::collections::VecDeque;

/// FIFO Ready Queue managing runnable tasks awaiting CPU allocation.
#[derive(Debug, Clone, Default)]
pub struct ReadyQueue {
    tasks: VecDeque<TaskId>,
}

impl ReadyQueue {
    pub fn new() -> Self {
        Self {
            tasks: VecDeque::new(),
        }
    }

    /// Enqueue a task to the tail of the ready queue.
    pub fn enqueue(&mut self, id: TaskId) {
        if !self.contains(id) {
            self.tasks.push_back(id);
        }
    }

    /// Dequeue the next task from the head of the ready queue.
    pub fn dequeue(&mut self) -> Option<TaskId> {
        self.tasks.pop_front()
    }

    /// Peek at the task currently at the head without removing it.
    pub fn peek(&self) -> Option<TaskId> {
        self.tasks.front().copied()
    }

    /// Remove a specific task from anywhere in the queue (e.g. if blocked or killed).
    pub fn remove(&mut self, id: TaskId) -> bool {
        if let Some(pos) = self.tasks.iter().position(|&t| t == id) {
            self.tasks.remove(pos);
            true
        } else {
            false
        }
    }

    /// Check if task is in the ready queue.
    pub fn contains(&self, id: TaskId) -> bool {
        self.tasks.contains(&id)
    }

    pub fn len(&self) -> usize {
        self.tasks.len()
    }

    pub fn is_empty(&self) -> bool {
        self.tasks.is_empty()
    }

    pub fn clear(&mut self) {
        self.tasks.clear();
    }

    /// Returns a snapshot of current tasks in queue order.
    pub fn list(&self) -> Vec<TaskId> {
        self.tasks.iter().copied().collect()
    }
}
