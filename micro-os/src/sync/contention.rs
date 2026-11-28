//! Lock Contention and Concurrency Scalability Analyzer.

use crate::sync::spinlock::LockStats;
use crate::sync::mutex::MutexStats;

/// High-level report summarizing synchronization behavior and overhead.
#[derive(Debug, Clone, Default)]
pub struct ContentionReport {
    pub lock_type: &'static str,
    pub active_tasks: usize,
    pub total_acquisitions: usize,
    pub contended_acquisitions: usize,
    pub contention_rate_percent: f64,
    pub total_wait_cycles: u64,
    pub avg_wait_cycles_per_contention: f64,
    pub max_wait_cycles: u64,
    pub throughput_acquisitions_per_kcycle: f64,
}

impl ContentionReport {
    pub fn from_spinlock_stats(name: &'static str, tasks: usize, stats: &LockStats, total_cycles: u64) -> Self {
        let contention_rate = stats.contention_rate();
        let avg_wait = stats.avg_spin_cycles();
        let throughput = if total_cycles > 0 {
            (stats.total_acquisitions as f64) / (total_cycles as f64) * 1000.0
        } else {
            0.0
        };

        Self {
            lock_type: name,
            active_tasks: tasks,
            total_acquisitions: stats.total_acquisitions,
            contended_acquisitions: stats.contended_acquisitions,
            contention_rate_percent: contention_rate,
            total_wait_cycles: stats.total_spin_cycles,
            avg_wait_cycles_per_contention: avg_wait,
            max_wait_cycles: stats.max_spin_cycles,
            throughput_acquisitions_per_kcycle: throughput,
        }
    }

    pub fn from_mutex_stats(name: &'static str, tasks: usize, stats: &MutexStats, total_cycles: u64) -> Self {
        let contention_rate = stats.contention_rate();
        let avg_blocked = if stats.tasks_blocked_count > 0 {
            (stats.total_blocked_ticks as f64) / (stats.tasks_blocked_count as f64)
        } else {
            0.0
        };
        let throughput = if total_cycles > 0 {
            (stats.total_acquisitions as f64) / (total_cycles as f64) * 1000.0
        } else {
            0.0
        };

        Self {
            lock_type: name,
            active_tasks: tasks,
            total_acquisitions: stats.total_acquisitions,
            contended_acquisitions: stats.contended_acquisitions,
            contention_rate_percent: contention_rate,
            total_wait_cycles: stats.total_blocked_ticks as u64,
            avg_wait_cycles_per_contention: avg_blocked,
            max_wait_cycles: stats.max_blocked_ticks as u64,
            throughput_acquisitions_per_kcycle: throughput,
        }
    }

    pub fn display_summary(&self) {
        println!("------------------------------------------------------------");
        println!(" CONCURRENCY EVALUATION REPORT: {}", self.lock_type);
        println!("------------------------------------------------------------");
        println!(" Active Tasks        : {}", self.active_tasks);
        println!(" Total Acquisitions  : {}", self.total_acquisitions);
        println!(" Contended Attempts  : {}", self.contended_acquisitions);
        println!(" Contention Rate     : {:.2}%", self.contention_rate_percent);
        println!(" Total Wait Overhead : {} cycles/ticks", self.total_wait_cycles);
        println!(" Avg Wait (Contended): {:.2} cycles/ticks", self.avg_wait_cycles_per_contention);
        println!(" Max Wait Overhead   : {} cycles/ticks", self.max_wait_cycles);
        println!(" Lock Throughput     : {:.2} ops / 1000 cycles", self.throughput_acquisitions_per_kcycle);
        println!("------------------------------------------------------------");
    }
}
