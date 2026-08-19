//! Memory management for LITHOS engine
//! Arena allocators, pool allocators, and global heap tracking

use std::alloc::{Layout, alloc, dealloc, handle_alloc_error};
use std::cell::UnsafeCell;
use std::marker::PhantomData;
use std::ptr::NonNull;
use std::sync::atomic::{AtomicUsize, Ordering};
use parking_lot::Mutex;
use tracing::{debug, info, warn};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum MemoryError {
    #[error("Out of memory: requested {requested} bytes, available {available}")]
    OutOfMemory { requested: usize, available: usize },
    #[error("Invalid layout: {0}")]
    InvalidLayout(String),
    #[error("Arena exhausted: {used}/{capacity} bytes used")]
    ArenaExhausted { used: usize, capacity: usize },
    #[error("Pool exhausted: no free slots")]
    PoolExhausted,
}

/// Global allocation statistics
pub struct GlobalStats {
    pub total_allocated: AtomicUsize,
    pub total_deallocated: AtomicUsize,
    pub current_allocated: AtomicUsize,
    pub peak_allocated: AtomicUsize,
    pub allocation_count: AtomicUsize,
    pub deallocation_count: AtomicUsize,
}

impl Default for GlobalStats {
    fn default() -> Self {
        Self::new()
    }
}

impl GlobalStats {
    pub const fn new() -> Self {
        Self {
            total_allocated: AtomicUsize::new(0),
            total_deallocated: AtomicUsize::new(0),
            current_allocated: AtomicUsize::new(0),
            peak_allocated: AtomicUsize::new(0),
            allocation_count: AtomicUsize::new(0),
            deallocation_count: AtomicUsize::new(0),
        }
    }

    pub fn record_alloc(&self, size: usize) {
        self.total_allocated.fetch_add(size, Ordering::Relaxed);
        self.allocation_count.fetch_add(1, Ordering::Relaxed);
        let current = self.current_allocated.fetch_add(size, Ordering::Relaxed) + size;
        let mut peak = self.peak_allocated.load(Ordering::Relaxed);
        while current > peak {
            match self.peak_allocated.compare_exchange_weak(
                peak,
                current,
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                Ok(_) => break,
                Err(x) => peak = x,
            }
        }
    }

    pub fn record_dealloc(&self, size: usize) {
        self.total_deallocated.fetch_add(size, Ordering::Relaxed);
        self.deallocation_count.fetch_add(1, Ordering::Relaxed);
        self.current_allocated.fetch_sub(size, Ordering::Relaxed);
    }

    pub fn current_bytes(&self) -> usize {
        self.current_allocated.load(Ordering::Relaxed)
    }

    pub fn peak_bytes(&self) -> usize {
        self.peak_allocated.load(Ordering::Relaxed)
    }

    pub fn total_allocated_bytes(&self) -> usize {
        self.total_allocated.load(Ordering::Relaxed)
    }

    pub fn allocation_count(&self) -> usize {
        self.allocation_count.load(Ordering::Relaxed)
    }
}

/// Global allocator tracker
pub static GLOBAL_STATS: GlobalStats = GlobalStats::new();

pub fn global_stats() -> &'static GlobalStats {
    &GLOBAL_STATS
}

/// Arena allocator for temporary, frame-scoped allocations
pub struct Arena {
    buffer: UnsafeCell<*mut u8>,
    capacity: usize,
    offset: UnsafeCell<usize>,
    name: &'static str,
}

impl Arena {
    pub fn new(capacity: usize, name: &'static str) -> Result<Self, MemoryError> {
        let layout = Layout::from_size_align(capacity, 64)
            .map_err(|e| MemoryError::InvalidLayout(e.to_string()))?;

        let ptr = unsafe { alloc(layout) };
        if ptr.is_null() {
            return Err(MemoryError::OutOfMemory {
                requested: capacity,
                available: 0,
            });
        }

        GLOBAL_STATS.record_alloc(capacity);

        Ok(Self {
            buffer: UnsafeCell::new(ptr),
            capacity,
            offset: UnsafeCell::new(0),
            name,
        })
    }

    pub fn allocate(&self, layout: Layout) -> Result<NonNull<u8>, MemoryError> {
        let align = layout.align();
        let size = layout.size();

        let offset_ptr = self.offset.get();
        let current_offset = unsafe { *offset_ptr };

        let aligned_offset = (current_offset + align - 1) & !(align - 1);
        let new_offset = aligned_offset + size;

        if new_offset > self.capacity {
            return Err(MemoryError::ArenaExhausted {
                used: current_offset,
                capacity: self.capacity,
            });
        }

        unsafe {
            *offset_ptr = new_offset;
            let base = *self.buffer.get();
            let ptr = base.add(aligned_offset);
            Ok(NonNull::new_unchecked(ptr))
        }
    }

    pub fn allocate_zeroed(&self, layout: Layout) -> Result<NonNull<u8>, MemoryError> {
        let ptr = self.allocate(layout)?;
        unsafe {
            std::ptr::write_bytes(ptr.as_ptr(), 0, layout.size());
        }
        Ok(ptr)
    }

    pub fn reset(&self) {
        unsafe {
            *self.offset.get() = 0;
        }
    }

    pub fn used(&self) -> usize {
        unsafe { *self.offset.get() }
    }

    pub fn remaining(&self) -> usize {
        self.capacity - self.used()
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }

    pub fn name(&self) -> &'static str {
        self.name
    }
}

impl Drop for Arena {
    fn drop(&mut self) {
        let ptr = unsafe { *self.buffer.get() };
        let layout = Layout::from_size_align(self.capacity, 64).unwrap();
        unsafe {
            dealloc(ptr, layout);
        }
        GLOBAL_STATS.record_dealloc(self.capacity);
    }
}

/// Pool allocator for fixed-size objects
pub struct Pool<T> {
    blocks: Mutex<Vec<*mut T>>,
    free_list: Mutex<Vec<*mut T>>,
    block_size: usize,
    blocks_per_chunk: usize,
    name: &'static str,
}

impl<T> Pool<T> {
    pub fn new(blocks_per_chunk: usize, name: &'static str) -> Self {
        let block_size = std::mem::size_of::<T>();
        Self {
            blocks: Mutex::new(Vec::new()),
            free_list: Mutex::new(Vec::new()),
            block_size,
            blocks_per_chunk,
            name,
        }
    }

    fn allocate_chunk(&self) -> Result<(), MemoryError> {
        let layout = Layout::array::<T>(self.blocks_per_chunk)
            .map_err(|e| MemoryError::InvalidLayout(e.to_string()))?;

        let ptr = unsafe { alloc(layout) };
        if ptr.is_null() {
            return Err(MemoryError::OutOfMemory {
                requested: layout.size(),
                available: 0,
            });
        }

        let chunk_bytes = layout.size();
        GLOBAL_STATS.record_alloc(chunk_bytes);

        let mut blocks = self.blocks.lock();
        let mut free = self.free_list.lock();

        blocks.push(ptr as *mut T);

        for i in 0..self.blocks_per_chunk {
            let block_ptr = unsafe { (ptr as *mut T).add(i) };
            free.push(block_ptr);
        }

        Ok(())
    }

    pub fn allocate(&self) -> Result<NonNull<T>, MemoryError> {
        let mut free = self.free_list.lock();
        if let Some(ptr) = free.pop() {
            return Ok(NonNull::new(ptr).unwrap());
        }
        drop(free);

        self.allocate_chunk()?;

        let mut free = self.free_list.lock();
        free.pop()
            .map(NonNull::new)
            .unwrap()
            .ok_or(MemoryError::PoolExhausted)
    }

    pub fn deallocate(&self, ptr: NonNull<T>) {
        let mut free = self.free_list.lock();
        free.push(ptr.as_ptr());
    }

    pub fn stats(&self) -> PoolStats {
        let blocks = self.blocks.lock();
        let free = self.free_list.lock();
        PoolStats {
            total_blocks: blocks.len() * self.blocks_per_chunk,
            free_blocks: free.len(),
            used_blocks: blocks.len() * self.blocks_per_chunk - free.len(),
            chunk_count: blocks.len(),
            bytes_per_block: self.block_size,
        }
    }
}

#[derive(Debug, Clone)]
pub struct PoolStats {
    pub total_blocks: usize,
    pub free_blocks: usize,
    pub used_blocks: usize,
    pub chunk_count: usize,
    pub bytes_per_block: usize,
}

impl<T> Drop for Pool<T> {
    fn drop(&mut self) {
        let blocks = self.blocks.lock();
        for block in blocks.iter() {
            let layout = Layout::array::<T>(self.blocks_per_chunk).unwrap();
            unsafe { dealloc(*block as *mut u8, layout) };
            GLOBAL_STATS.record_dealloc(layout.size());
        }
    }
}

/// Scoped arena that automatically resets on drop
pub struct ScopedArena<'a> {
    arena: &'a Arena,
    checkpoint: usize,
}

impl<'a> ScopedArena<'a> {
    pub fn new(arena: &'a Arena) -> Self {
        let checkpoint = arena.used();
        Self { arena, checkpoint }
    }

    pub fn allocate(&self, layout: Layout) -> Result<NonNull<u8>, MemoryError> {
        self.arena.allocate(layout)
    }

    pub fn allocate_zeroed(&self, layout: Layout) -> Result<NonNull<u8>, MemoryError> {
        self.arena.allocate_zeroed(layout)
    }
}

impl<'a> Drop for ScopedArena<'a> {
    fn drop(&mut self) {
        self.arena.reset();
    }
}

/// Allocation guard for debugging
#[cfg(debug_assertions)]
pub struct AllocationGuard {
    name: &'static str,
    expected_count: usize,
}

#[cfg(debug_assertions)]
impl AllocationGuard {
    pub fn new(name: &'static str) -> Self {
        Self {
            name,
            expected_count: GLOBAL_STATS.allocation_count(),
        }
    }
}

#[cfg(debug_assertions)]
impl Drop for AllocationGuard {
    fn drop(&mut self) {
        let current = GLOBAL_STATS.allocation_count();
        if current != self.expected_count {
            warn!(
                "Allocation count mismatch in '{}': expected {}, got {}",
                self.name, self.expected_count, current
            );
        }
    }
}

#[cfg(not(debug_assertions))]
pub struct AllocationGuard;

#[cfg(not(debug_assertions))]
impl AllocationGuard {
    pub fn new(_name: &'static str) -> Self {
        Self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_arena_basic() {
        let arena = Arena::new(1024, "test").unwrap();
        let layout = Layout::new::<u64>();
        let ptr = arena.allocate(layout).unwrap();
        assert!(!ptr.as_ptr().is_null());
        assert_eq!(arena.used(), 8);
    }

    #[test]
    fn test_arena_reset() {
        let arena = Arena::new(1024, "test").unwrap();
        let layout = Layout::new::<u64>();
        arena.allocate(layout).unwrap();
        arena.reset();
        assert_eq!(arena.used(), 0);
    }

    #[test]
    fn test_pool_basic() {
        let pool = Pool::<u64>::new(16, "test");
        let ptr = pool.allocate().unwrap();
        assert!(!ptr.as_ptr().is_null());
        let stats = pool.stats();
        assert_eq!(stats.used_blocks, 1);
        pool.deallocate(ptr);
        let stats = pool.stats();
        assert_eq!(stats.free_blocks, 1);
    }

    #[test]
    fn test_scoped_arena() {
        let arena = Arena::new(1024, "test").unwrap();
        {
            let scoped = ScopedArena::new(&arena);
            let layout = Layout::new::<u64>();
            scoped.allocate(layout).unwrap();
        }
        assert_eq!(arena.used(), 0);
    }
}