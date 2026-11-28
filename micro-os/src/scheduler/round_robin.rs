//! Round Robin Scheduler and Kernel Runtime Engine.

use crate::kernel::cpu::CpuContext;
use crate::kernel::logger::{kdebug, kerror, kinfo, ktrace, kwarn, set_tick};
use crate::memory::{BlockAllocator, BumpAllocator};
use crate::scheduler::queue::ReadyQueue;
use crate::sync::{DeadlockManager, MutexStats, SleepMutex, Spinlock, TicketSpinlock};
use crate::task::{TaskControlBlock, TaskId, TaskInstruction, TaskState};
use std::collections::HashMap;

/// Trace event emitted by the scheduler for empirical observation.
#[derive(Debug, Clone)]
pub struct SchedulerEvent {
    pub tick: usize,
    pub event_type: &'static str,
    pub task_id: Option<TaskId>,
    pub details: String,
}

/// The Core Kernel Engine coordinating CPU, Scheduling, Memory, and Synchronization.
pub struct KernelEngine {
    pub quantum_ticks: usize,
    pub current_quantum_used: usize,
    pub current_task: Option<TaskId>,
    pub ready_queue: ReadyQueue,
    pub tasks: HashMap<TaskId, TaskControlBlock>,
    pub spinlocks: HashMap<usize, Spinlock>,
    pub ticket_locks: HashMap<usize, TicketSpinlock>,
    pub mutexes: HashMap<usize, SleepMutex>,
    pub deadlock_mgr: DeadlockManager,
    pub bump_allocator: BumpAllocator,
    pub block_allocator: BlockAllocator,
    pub current_tick: usize,
    pub scheduler_events: Vec<SchedulerEvent>,
    pub total_context_switches: usize,
}

impl KernelEngine {
    pub fn new(quantum_ticks: usize, heap_size_bytes: usize) -> Self {
        Self {
            quantum_ticks,
            current_quantum_used: 0,
            current_task: None,
            ready_queue: ReadyQueue::new(),
            tasks: HashMap::new(),
            spinlocks: HashMap::new(),
            ticket_locks: HashMap::new(),
            mutexes: HashMap::new(),
            deadlock_mgr: DeadlockManager::new(),
            bump_allocator: BumpAllocator::new(heap_size_bytes / 2),
            block_allocator: BlockAllocator::new(heap_size_bytes / 2),
            current_tick: 0,
            scheduler_events: Vec::new(),
            total_context_switches: 0,
        }
    }

    /// Register a new task into the kernel.
    pub fn spawn_task(&mut self, mut task: TaskControlBlock) {
        let id = task.id;
        task.state = TaskState::Ready;
        task.metrics.arrival_tick = self.current_tick;
        kinfo!("SCHED", "Spawning task {} ('{}', priority={})", id, task.name, task.priority);
        self.log_event("TASK_SPAWN", Some(id), format!("Priority {}", task.priority));
        self.tasks.insert(id, task);
        self.ready_queue.enqueue(id);
    }

    /// Register a spinlock with a given rank for deadlock prevention.
    pub fn register_spinlock(&mut self, id: usize, name: &'static str, rank: u32) {
        self.spinlocks.insert(id, Spinlock::new(id, name, rank));
    }

    /// Register a sleeping mutex with a given rank for deadlock prevention.
    pub fn register_mutex(&mut self, id: usize, name: &'static str, rank: u32) {
        self.mutexes.insert(id, SleepMutex::new(id, name, rank));
    }

    /// Perform a context switch to the next ready task.
    pub fn context_switch(&mut self) {
        let prev_task_id = self.current_task.take();

        // Save state of previous task if it was running
        if let Some(pid) = prev_task_id {
            if let Some(ptask) = self.tasks.get_mut(&pid) {
                if ptask.state == TaskState::Running {
                    ptask.state = TaskState::Ready;
                    self.ready_queue.enqueue(pid);
                    self.log_event("PREEMPT", Some(pid), "Quantum elapsed or voluntary switch".to_string());
                }
            }
        }

        // Dequeue next runnable task
        if let Some(next_id) = self.ready_queue.dequeue() {
            if let Some(ntask) = self.tasks.get_mut(&next_id) {
                ntask.state = TaskState::Running;
                if ntask.metrics.start_tick.is_none() {
                    ntask.metrics.start_tick = Some(self.current_tick);
                }
                ntask.metrics.context_switches += 1;
                self.current_task = Some(next_id);
                self.current_quantum_used = 0;
                self.total_context_switches += 1;

                self.log_event(
                    "DISPATCH",
                    Some(next_id),
                    format!("Restored context SP=0x{:04x}", ntask.context.sp),
                );
                ktrace!("SCHED", "Context switched to task {}", next_id);
            }
        } else {
            self.current_task = None;
            ktrace!("SCHED", "Ready queue empty; CPU idling");
        }
    }

    /// Execute one discrete kernel tick.
    pub fn step(&mut self) -> bool {
        self.current_tick += 1;
        set_tick(self.current_tick);

        // If no task is currently dispatched, attempt to dispatch one
        if self.current_task.is_none() {
            self.context_switch();
        }

        // If still no task, all tasks are blocked or finished
        let current_id = match self.current_task {
            Some(id) => id,
            None => {
                // Check if any tasks are still runnable or blocked
                return self.has_unfinished_tasks();
            }
        };

        // Execute task instruction
        let mut reschedule_needed = false;
        let mut task_terminated = false;

        if let Some(task) = self.tasks.get_mut(&current_id) {
            if let Some(instr) = task.next_instruction().cloned() {
                match instr {
                    TaskInstruction::Compute(cycles) => {
                        task.context.cycles_executed += cycles;
                        task.metrics.total_cpu_cycles += cycles;
                        task.advance_instruction();
                        ktrace!("TASK", "{} computed for {} cycles", current_id, cycles);
                    }
                    TaskInstruction::SpinlockAcquire(lock_id) => {
                        let rank = self.spinlocks.get(&lock_id).map(|l| l.rank()).unwrap_or(0);
                        // Deadlock prevention check
                        if let Err(e) = self.deadlock_mgr.validate_acquisition(current_id, lock_id, rank) {
                            kerror!("SYNC", "{}", e);
                            self.log_event("DEADLOCK_PREVENTED", Some(current_id), format!("{}", e));
                            // Advance to prevent infinite loop on failure test case
                            task.advance_instruction();
                        } else if let Some(lock) = self.spinlocks.get_mut(&lock_id) {
                            let (acquired, spins) = lock.acquire_with_spin(current_id, 10);
                            task.metrics.lock_wait_cycles += spins;
                            if acquired {
                                self.deadlock_mgr.record_acquired(current_id, lock_id, rank);
                                task.metrics.locks_acquired += 1;
                                task.advance_instruction();
                                kdebug!("SYNC", "{} acquired Spinlock-{} after {} spins", current_id, lock_id, spins);
                            } else {
                                // Lock contested; spin in next tick
                                ktrace!("SYNC", "{} spinning on Spinlock-{}", current_id, lock_id);
                            }
                        } else {
                            kwarn!("SYNC", "Spinlock-{} not found", lock_id);
                            task.advance_instruction();
                        }
                    }
                    TaskInstruction::SpinlockRelease(lock_id) => {
                        if let Some(lock) = self.spinlocks.get_mut(&lock_id) {
                            if let Ok(()) = lock.release(current_id) {
                                self.deadlock_mgr.record_released(current_id, lock_id);
                                task.advance_instruction();
                                kdebug!("SYNC", "{} released Spinlock-{}", current_id, lock_id);
                            }
                        } else {
                            task.advance_instruction();
                        }
                    }
                    TaskInstruction::MutexLock(mutex_id) => {
                        let rank = self.mutexes.get(&mutex_id).map(|m| m.rank()).unwrap_or(0);
                        // Deadlock prevention check
                        if let Err(e) = self.deadlock_mgr.validate_acquisition(current_id, mutex_id, rank) {
                            kerror!("SYNC", "{}", e);
                            self.log_event("DEADLOCK_PREVENTED", Some(current_id), format!("{}", e));
                            task.advance_instruction();
                        } else if let Some(mutex) = self.mutexes.get_mut(&mutex_id) {
                            if mutex.try_lock(current_id) {
                                self.deadlock_mgr.record_acquired(current_id, mutex_id, rank);
                                task.metrics.locks_acquired += 1;
                                task.advance_instruction();
                                kdebug!("SYNC", "{} acquired Mutex-{}", current_id, mutex_id);
                            } else {
                                // Contended: Block task on mutex wait queue
                                mutex.block_task(current_id, self.current_tick);
                                task.state = TaskState::Blocked { reason: "Sleeping on Mutex" };
                                kinfo!("SYNC", "{} BLOCKED on Mutex-{} -> voluntary switch", current_id, mutex_id);
                                self.log_event("TASK_BLOCKED", Some(current_id), format!("Mutex-{}", mutex_id));
                                reschedule_needed = true;
                            }
                        } else {
                            task.advance_instruction();
                        }
                    }
                    TaskInstruction::MutexUnlock(mutex_id) => {
                        let current_tick = self.current_tick;
                        if let Some(mutex) = self.mutexes.get_mut(&mutex_id) {
                            match mutex.unlock(current_id, current_tick) {
                                Ok(Some((woken_task, blocked_duration))) => {
                                    self.deadlock_mgr.record_released(current_id, mutex_id);
                                    let rank = mutex.rank();
                                    self.deadlock_mgr.record_acquired(woken_task, mutex_id, rank);
                                    
                                    // Wake up waiting task
                                    if let Some(wtask) = self.tasks.get_mut(&woken_task) {
                                        wtask.state = TaskState::Ready;
                                        wtask.metrics.lock_wait_cycles += blocked_duration as u64;
                                        wtask.metrics.locks_acquired += 1;
                                        wtask.advance_instruction(); // Acquired lock directly
                                        self.ready_queue.enqueue(woken_task);
                                        kinfo!("SYNC", "Mutex-{} transferred: {} unblocked after {} ticks", mutex_id, woken_task, blocked_duration);
                                        self.log_event("TASK_UNBLOCKED", Some(woken_task), format!("Mutex-{}", mutex_id));
                                    }
                                    task.advance_instruction();
                                }
                                Ok(None) => {
                                    self.deadlock_mgr.record_released(current_id, mutex_id);
                                    task.advance_instruction();
                                    kdebug!("SYNC", "{} unlocked Mutex-{} (no waiters)", current_id, mutex_id);
                                }
                                Err(err) => {
                                    kwarn!("SYNC", "{}", err);
                                    task.advance_instruction();
                                }
                            }
                        } else {
                            task.advance_instruction();
                        }
                    }
                    TaskInstruction::Allocate(bytes) => {
                        match self.bump_allocator.allocate(bytes) {
                            Ok(addr) => {
                                kdebug!("MEM", "{} allocated {} bytes at 0x{:04x}", current_id, bytes, addr);
                            }
                            Err(err) => {
                                kwarn!("MEM", "{} allocation failed ({} bytes): {}", current_id, bytes, err);
                            }
                        }
                        task.advance_instruction();
                    }
                    TaskInstruction::Yield => {
                        task.advance_instruction();
                        reschedule_needed = true;
                        kdebug!("SCHED", "{} voluntarily yielded CPU", current_id);
                    }
                    TaskInstruction::Exit => {
                        task.state = TaskState::Terminated;
                        task.metrics.finish_tick = Some(self.current_tick);
                        task_terminated = true;
                        kinfo!("TASK", "{} TERMINATED at tick {}", current_id, self.current_tick);
                        self.log_event("TASK_TERMINATED", Some(current_id), "Execution finished".to_string());
                    }
                }

                // If instructions ended without explicit Exit, terminate
                if task.is_completed() && task.state != TaskState::Terminated {
                    task.state = TaskState::Terminated;
                    task.metrics.finish_tick = Some(self.current_tick);
                    task_terminated = true;
                    kinfo!("TASK", "{} completed all instructions and TERMINATED", current_id);
                    self.log_event("TASK_TERMINATED", Some(current_id), "Completed all instructions".to_string());
                }
            } else {
                task.state = TaskState::Terminated;
                task.metrics.finish_tick = Some(self.current_tick);
                task_terminated = true;
            }
        }

        // Handle preemption or task termination
        self.current_quantum_used += 1;
        if task_terminated {
            self.current_task = None;
            self.context_switch();
        } else if reschedule_needed {
            self.context_switch();
        } else if self.current_quantum_used >= self.quantum_ticks {
            // Timer-driven Preemption
            if let Some(task) = self.tasks.get_mut(&current_id) {
                task.metrics.timeslices_exhausted += 1;
            }
            ktrace!("SCHED", "Quantum expired for {}; triggering preemption", current_id);
            self.context_switch();
        }

        self.has_unfinished_tasks()
    }

    /// Run the kernel loop until all tasks terminate or max_ticks is reached.
    pub fn run_until_complete(&mut self, max_ticks: usize) {
        kinfo!("SCHED", "Beginning execution loop (Quantum = {} ticks)", self.quantum_ticks);
        while self.step() && self.current_tick < max_ticks {}
        kinfo!("SCHED", "Kernel loop finished at tick {}", self.current_tick);
    }

    pub fn has_unfinished_tasks(&self) -> bool {
        self.tasks.values().any(|t| t.state != TaskState::Terminated)
    }

    pub fn log_event(&mut self, event_type: &'static str, task_id: Option<TaskId>, details: String) {
        self.scheduler_events.push(SchedulerEvent {
            tick: self.current_tick,
            event_type,
            task_id,
            details,
        });
    }

    pub fn print_metrics_summary(&self) {
        println!("\n============================================================");
        println!("             KERNEL EXECUTION METRICS SUMMARY               ");
        println!("============================================================");
        println!("Total Ticks Elapsed      : {}", self.current_tick);
        println!("Total Context Switches   : {}", self.total_context_switches);
        println!("Deadlocks Prevented      : {}", self.deadlock_mgr.violations_count());
        println!("Heap Usage               : {}", self.bump_allocator.stats());
        println!("------------------------------------------------------------");
        println!("{:<8} {:<12} {:<10} {:<10} {:<12} {:<8}", "TASK", "STATE", "CYCLES", "SWITCHES", "LOCK_WAITS", "SLICES");
        println!("------------------------------------------------------------");
        for (id, task) in &self.tasks {
            println!(
                "{:<8} {:<12} {:<10} {:<10} {:<12} {:<8}",
                id.to_string(),
                task.state.to_string(),
                task.metrics.total_cpu_cycles,
                task.metrics.context_switches,
                task.metrics.lock_wait_cycles,
                task.metrics.timeslices_exhausted,
            );
        }
        println!("============================================================\n");
    }
}
