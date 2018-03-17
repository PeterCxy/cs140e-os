use std::fmt;
use alloc::heap::{AllocErr, Layout};

use allocator::util::*;
use allocator::linked_list::LinkedList;

#[inline(always)]
fn calc_bin_size(bin: usize) -> usize {
    1 << (bin + 3)
}

/// A simple allocator that allocates based on size classes.
pub struct Allocator {
    bins: [LinkedList; 63 - 2], // at most 2^63, but we start as bins of 2^3
    bin_num: usize,
    free_start: usize,
    free_end: usize
}

impl Allocator {
    /// Creates a new bin allocator that will allocate memory from the region
    /// starting at address `start` and ending at address `end`.
    pub fn new(start: usize, end: usize) -> Allocator {
        Allocator {
            bins: [LinkedList::new(); 63 - 2],
            bin_num: log2_ceil(end - start) - 1 - 2,
            free_start: start,
            free_end: end
        }
    }

    // Naively allocate new chunk from unallocated memory
    fn naive_alloc(&mut self, len: usize, layout: Layout) -> Result<*mut u8, AllocErr> {
        let aligned_start = align_up(self.free_start, layout.align());
        if self.free_end.saturating_sub(aligned_start) < len {
            return Err(AllocErr::Exhausted { request: layout });
        } else {
            self.free_start = aligned_start + len;
            return Ok(aligned_start as *mut u8);
        }
    }

    // Split a free memory chunk from a larger bin to smaller bins starting from `start_bin`
    // this is used to allocate a smaller chunk from a bin in a higher rank
    fn split_free_memory(&mut self, start: usize, orig_bin: usize, start_bin: usize) {
        let mut cur_start = start;
        let mut cur_bin = start_bin;
        while cur_bin < orig_bin {
            unsafe {
                self.bins[cur_bin].push(cur_start as *mut usize);
            }
            cur_start += calc_bin_size(cur_bin);
            cur_bin += 1;
        }
    }

    // Insert a chunk into its corresponding bin
    // but try to merge the chunk with adjacent chunks if available
    unsafe fn coalesce_insert_chunk(&mut self, bin_index: usize, chunk: *mut usize) {
        if bin_index < self.bin_num - 1 {
            let bin_size = calc_bin_size(bin_index);
            let chunk_addr = chunk as usize;
            let mut merge_addr: Option<usize> = None;
            for node in self.bins[bin_index].iter_mut() {
                // Merge adjacent chunks if possible
                let addr = node.value() as usize;
                if addr > chunk_addr && addr - chunk_addr == bin_size {
                    merge_addr = Some(chunk_addr);
                    node.pop();
                    break;
                } else if addr < chunk_addr && chunk_addr - addr == bin_size {
                    merge_addr = Some(addr);
                    node.pop();
                    break;
                }
            }

            if let Some(addr) = merge_addr {
                self.coalesce_insert_chunk(bin_index + 1, addr as *mut usize);
                return;
            }
        }

        self.bins[bin_index].push(chunk);
    }

    /// Allocates memory. Returns a pointer meeting the size and alignment
    /// properties of `layout.size()` and `layout.align()`.
    ///
    /// If this method returns an `Ok(addr)`, `addr` will be non-null address
    /// pointing to a block of storage suitable for holding an instance of
    /// `layout`. In particular, the block will be at least `layout.size()`
    /// bytes large and will be aligned to `layout.align()`. The returned block
    /// of storage may or may not have its contents initialized or zeroed.
    ///
    /// # Safety
    ///
    /// The _caller_ must ensure that `layout.size() > 0` and that
    /// `layout.align()` is a power of two. Parameters not meeting these
    /// conditions may result in undefined behavior.
    ///
    /// # Errors
    ///
    /// Returning `Err` indicates that either memory is exhausted
    /// (`AllocError::Exhausted`) or `layout` does not meet this allocator's
    /// size or alignment constraints (`AllocError::Unsupported`).
    pub fn alloc(&mut self, layout: Layout) -> Result<*mut u8, AllocErr> {
        if !is_power_of_two(layout.align()) {
            return Err(AllocErr::Unsupported { details: "Only alignment to powers of 2 are supported" });
        }

        let bin_index = log2_ceil(layout.size()).saturating_sub(3);
        if bin_index > self.bin_num {
            return Err(AllocErr::Exhausted { request: layout });
        } else if bin_index == self.bin_num {
            // Requested to allocate over half of the total memory
            // We can only find something in the wilderness
            // TODO: Implement this
            // This might need another list to record all the wild memory
            return Err(AllocErr::Unsupported { details: "Cannot allocate so much memory" });
        } else {
            let bin_size = calc_bin_size(bin_index);
            let mut ret: Option<*mut u8> = None;
            let mut fin_index = 0;
            // Try to find a block that satisfies the alignment
            // We can also allocate from bins that are larger than the current one
            // by splitting it into smaller chunks
            'outer_loop: for index in bin_index..(self.bin_num) {
                for node in self.bins[index].iter_mut() {
                    let addr = node.value() as usize;

                    // Only return the free block if it applies to the alignment requirements
                    // If not, we'd better allocate some new memory for it.
                    if addr % layout.align() == 0 {
                        ret = Some(node.pop() as *mut u8);
                        fin_index = index;
                        break 'outer_loop;
                    }
                }
            }

            if let Some(ret) = ret {
                let addr = ret as usize;
                if fin_index != bin_index {
                    // We found a larger bin with proper alignment
                    // let's just devide it down
                    self.split_free_memory(addr + bin_size, fin_index, bin_index);
                }
                return Ok(ret);
            } else {
                return self.naive_alloc(bin_size, layout);
            }
        }
    }

    /// Deallocates the memory referenced by `ptr`.
    ///
    /// # Safety
    ///
    /// The _caller_ must ensure the following:
    ///
    ///   * `ptr` must denote a block of memory currently allocated via this
    ///     allocator
    ///   * `layout` must properly represent the original layout used in the
    ///     allocation call that returned `ptr`
    ///
    /// Parameters not meeting these conditions may result in undefined
    /// behavior.
    pub fn dealloc(&mut self, ptr: *mut u8, layout: Layout) {
        let bin_index = log2_ceil(layout.size()).saturating_sub(3);
        let bin_size = calc_bin_size(bin_index);
        if bin_index < self.bin_num {
            unsafe {
                self.coalesce_insert_chunk(bin_index, ptr as *mut usize);
                //self.bins[bin_index].push(ptr as *mut usize);
            }
        } else {
            // The other cases are not implemented in alloc() yet
            // so it is impossible to happen
            unimplemented!();
        }
    }
}

impl fmt::Debug for Allocator {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "---- UNALLOCATED MEMORY ----")?;
        writeln!(f, "FROM: {}", self.free_start)?;
        writeln!(f, "TO: {}", self.free_end)?;
        writeln!(f, "----   ALLOCATED BINS   ----")?;
        writeln!(f, "NUM: {}", self.bin_num)?;
        for i in 0..self.bin_num {
            write!(f, "BIN: {}, ", i)?;
            write!(f, "SIZE: {}, ", calc_bin_size(i))?;
            if let Some(head) = self.bins[i].peek() {
                let mut i = 0;
                let mut current_head = head;
                while !current_head.is_null() {
                    current_head = unsafe { *current_head as *mut usize };
                    i += 1;
                }
                writeln!(f, "ALLOCATED CHUNKS: {}", i);
            } else {
                writeln!(f, "EMPTY BIN");
            }
        }
        Ok(())
    }
}