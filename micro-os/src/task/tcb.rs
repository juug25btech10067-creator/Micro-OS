//! Task Control Block (TCB) and Task Representation.

use crate::kernel::cpu::CpuContext;
use std::fmt;

/// Unique Task Identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TaskId(pub usize);

impl fmt::Display for TaskId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Task-{}", self.0)
    }
}

/// Lifecycle states of a task in Micro-OS.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskState {
    Ready,
    Running,
    Blocked { reason: &'static str },
    Terminated,
}

impl fmt::Display for TaskState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TaskState::Ready => write!(f, "READY"),
            TaskState::Running => write!(f, "RUNNING"),
            TaskState::Blocked { reason } => write!(f, "BLOCKED({})", reason),
            TaskState::Terminated => write!(f, "TERMINATED"),
        }
    }
}

/// Task execution statistics for empirical performance evaluation.
#[derive(Debug, Clone, Default)]
pub struct TaskMetrics {
    pub arrival_tick: usize,
    pub start_tick: Option<usize>,
    pub finish_tick: Option<usize>,
    pub total_cpu_cycles: u64,
    pub total_wait_cycles: u64,
    pub context_switches: usize,
    pub timeslices_exhausted: usize,
    pub locks_acquired: usize,
    pub lock_wait_cycles: u64,
}

/// Simulated task virtual stack.
#[derive(Debug, Clone)]
pub struct VirtualStack {
    pub base: usize,
    pub size: usize,
    pub sp: usize,
    pub memory: Vec<u8>,
}

impl VirtualStack {
    pub fn new(base: usize, size: usize) -> Self {
        Self {
            base,
            size,
            sp: base + size,
            memory: vec![0; size],
        }
    }

    pub fn check_bounds(&self) -> Result<(), &'static str> {
        if self.sp < self.base {
            Err("Stack Overflow: SP lower than stack base")
        } else if self.sp > self.base + self.size {
            Err("Stack Underflow: SP beyond stack top")
        } else {
            Ok(())
        }
    }
}

/// Discrete steps a task can execute in this educational kernel.
#[derive(Debug, Clone)]
pub enum TaskInstruction {
    /// Pure computation consuming a specified number of CPU cycles
    Compute(u64),
    /// Request to acquire a spinlock by ID
    SpinlockAcquire(usize),
    /// Release a spinlock by ID
    SpinlockRelease(usize),
    /// Request to acquire a sleeping mutex by ID
    MutexLock(usize),
    /// Release a sleeping mutex by ID
    MutexUnlock(usize),
    /// Request memory allocation of N bytes
    Allocate(usize),
    /// Voluntarily yield the remaining time quantum
    Yield,
    /// Terminate task execution
    Exit,
}

/// Task Control Block containing all architectural metadata.
pub struct TaskControlBlock {
    pub id: TaskId,
    pub name: String,
    pub state: TaskState,
    pub priority: u8,
    pub context: CpuContext,
    pub stack: VirtualStack,
    pub instructions: Vec<TaskInstruction>,
    pub current_instruction_index: usize,
    pub metrics: TaskMetrics,
}

impl TaskControlBlock {
    pub fn new(
        id: TaskId,
        name: &str,
        priority: u8,
        instructions: Vec<TaskInstruction>,
        stack_base: usize,
        stack_size: usize,
        arrival_tick: usize,
    ) -> Self {
        let stack = VirtualStack::new(stack_base, stack_size);
        let stack_top = stack.sp;

        Self {
            id,
            name: name.to_string(),
            state: TaskState::Ready,
            priority,
            context: CpuContext::new(0x0000, stack_top),
            stack,
            instructions,
            current_instruction_index: 0,
            metrics: TaskMetrics {
                arrival_tick,
                ..Default::default()
            },
        }
    }

    /// Fetch next instruction to execute.
    pub fn next_instruction(&self) -> Option<&TaskInstruction> {
        self.instructions.get(self.current_instruction_index)
    }

    /// Advance instruction pointer upon successful execution.
    pub fn advance_instruction(&mut self) {
        self.current_instruction_index += 1;
        self.context.pc = self.current_instruction_index;
    }

    /// Check if task has completed all instructions.
    pub fn is_completed(&self) -> bool {
        self.current_instruction_index >= self.instructions.len()
    }
}
