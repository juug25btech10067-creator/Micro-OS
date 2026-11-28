//! Bump (watermark) allocator for educational kernel allocations.

use crate::memory::tracker::MemoryStats;

pub struct BumpAllocator {
    heap: Vec<u8>,
    next_offset: usize,
    stats: MemoryStats,
}

impl BumpAllocator {
    pub fn new(size_bytes: usize) -> Self {
        Self {
            heap: vec![0; size_bytes],
            next_offset: 0,
            stats: MemoryStats::new(size_bytes),
        }
    }

    /// Allocate `size` bytes aligned to 8-byte boundary.
    pub fn allocate(&mut self, size: usize) -> Result<usize, &'static str> {
        if size == 0 {
            return Ok(self.next_offset); // TC3: Boundary Case 0 bytes
        }

        // 8-byte alignment
        let aligned_offset = (self.next_offset + 7) & !7;
        let end_offset = aligned_offset.checked_add(size).ok_or("Integer overflow in allocation")?;

        if end_offset > self.heap.len() {
            self.stats.record_oom();
            return Err("Out of Memory (OOM): Bump allocator heap exhausted");
        }

        self.next_offset = end_offset;
        self.stats.record_allocation(size);
        Ok(aligned_offset)
    }

    /// Reset watermark (bulk deallocation).
    pub fn reset(&mut self) {
        let freed = self.stats.allocated_bytes;
        self.next_offset = 0;
        self.stats.record_deallocation(freed);
    }

    pub fn stats(&self) -> &MemoryStats {
        &self.stats
    }

    pub fn stats_mut(&mut self) -> &mut MemoryStats {
        &mut self.stats
    }
}
