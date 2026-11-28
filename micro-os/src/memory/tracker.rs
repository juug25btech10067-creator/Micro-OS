//! Kernel Memory Accounting and Diagnostics.

use std::fmt;

#[derive(Debug, Clone, Default)]
pub struct MemoryStats {
    pub total_heap_bytes: usize,
    pub allocated_bytes: usize,
    pub peak_allocated_bytes: usize,
    pub allocation_requests: usize,
    pub deallocation_requests: usize,
    pub out_of_memory_errors: usize,
}

impl MemoryStats {
    pub fn new(total_heap_bytes: usize) -> Self {
        Self {
            total_heap_bytes,
            ..Default::default()
        }
    }

    pub fn record_allocation(&mut self, bytes: usize) {
        self.allocated_bytes += bytes;
        self.allocation_requests += 1;
        if self.allocated_bytes > self.peak_allocated_bytes {
            self.peak_allocated_bytes = self.allocated_bytes;
        }
    }

    pub fn record_deallocation(&mut self, bytes: usize) {
        self.allocated_bytes = self.allocated_bytes.saturating_sub(bytes);
        self.deallocation_requests += 1;
    }

    pub fn record_oom(&mut self) {
        self.out_of_memory_errors += 1;
    }

    pub fn utilization_rate(&self) -> f64 {
        if self.total_heap_bytes == 0 {
            0.0
        } else {
            (self.allocated_bytes as f64) / (self.total_heap_bytes as f64) * 100.0
        }
    }

    pub fn peak_utilization_rate(&self) -> f64 {
        if self.total_heap_bytes == 0 {
            0.0
        } else {
            (self.peak_allocated_bytes as f64) / (self.total_heap_bytes as f64) * 100.0
        }
    }
}

impl fmt::Display for MemoryStats {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Heap: {} B | Used: {} B ({:.1}%) | Peak: {} B ({:.1}%) | Allocs: {} | Frees: {} | OOMs: {}",
            self.total_heap_bytes,
            self.allocated_bytes,
            self.utilization_rate(),
            self.peak_allocated_bytes,
            self.peak_utilization_rate(),
            self.allocation_requests,
            self.deallocation_requests,
            self.out_of_memory_errors
        )
    }
}
