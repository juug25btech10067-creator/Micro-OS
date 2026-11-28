//! Fixed-size Block Allocator with Free-List recycling.

use crate::memory::tracker::MemoryStats;

pub const BLOCK_SIZE: usize = 64; // 64-byte educational block granularity

pub struct BlockAllocator {
    _raw_memory: Vec<u8>,
    num_blocks: usize,
    free_list: Vec<usize>, // Stores block indices
    allocated_map: Vec<bool>,
    stats: MemoryStats,
}

impl BlockAllocator {
    pub fn new(total_bytes: usize) -> Self {
        let num_blocks = total_bytes / BLOCK_SIZE;
        let mut free_list = Vec::with_capacity(num_blocks);
        for i in (0..num_blocks).rev() {
            free_list.push(i);
        }

        Self {
            _raw_memory: vec![0; num_blocks * BLOCK_SIZE],
            num_blocks,
            free_list,
            allocated_map: vec![false; num_blocks],
            stats: MemoryStats::new(num_blocks * BLOCK_SIZE),
        }
    }

    /// Allocate one fixed-size block.
    pub fn allocate_block(&mut self) -> Result<usize, &'static str> {
        if let Some(block_idx) = self.free_list.pop() {
            self.allocated_map[block_idx] = true;
            self.stats.record_allocation(BLOCK_SIZE);
            Ok(block_idx * BLOCK_SIZE)
        } else {
            self.stats.record_oom();
            Err("Out of Memory (OOM): Free-list blocks exhausted")
        }
    }

    /// Deallocate a previously allocated block, returning it to the free-list.
    pub fn deallocate_block(&mut self, address: usize) -> Result<(), &'static str> {
        if address % BLOCK_SIZE != 0 {
            return Err("Deallocation Error: Unaligned block address");
        }
        let block_idx = address / BLOCK_SIZE;
        if block_idx >= self.num_blocks {
            return Err("Deallocation Error: Address out of pool bounds");
        }
        if !self.allocated_map[block_idx] {
            return Err("Deallocation Error: Double-free detected on block");
        }

        self.allocated_map[block_idx] = false;
        self.free_list.push(block_idx);
        self.stats.record_deallocation(BLOCK_SIZE);
        Ok(())
    }

    pub fn free_blocks_count(&self) -> usize {
        self.free_list.len()
    }

    pub fn total_blocks_count(&self) -> usize {
        self.num_blocks
    }

    pub fn stats(&self) -> &MemoryStats {
        &self.stats
    }
}
