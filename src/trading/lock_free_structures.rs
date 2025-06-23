// Lock-Free Data Structures for High-Performance Trading
// Implements atomic operations and lock-free data structures for maximum throughput

use crate::trading::signals::OrderRequest;
use chrono::Utc;
use serde::{Serialize, Deserialize};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, AtomicUsize, AtomicPtr, Ordering};
use std::ptr;
use tracing::debug;

/// Lock-free order book entry using atomic operations
#[derive(Debug)]
pub struct LockFreeOrderBookEntry {
    pub price: AtomicU64, // Store as fixed-point integer for atomic operations
    pub size: AtomicU64,
    pub order_count: AtomicUsize,
    pub timestamp: AtomicU64, // Store as nanoseconds since epoch
}

impl LockFreeOrderBookEntry {
    pub fn new(price: f64, size: f64, order_count: usize) -> Self {
        Self {
            price: AtomicU64::new((price * 1_000_000.0) as u64), // 6 decimal places precision
            size: AtomicU64::new((size * 1_000_000.0) as u64),
            order_count: AtomicUsize::new(order_count),
            timestamp: AtomicU64::new(Utc::now().timestamp_nanos_opt().unwrap_or(0) as u64),
        }
    }

    pub fn get_price(&self) -> f64 {
        self.price.load(Ordering::Relaxed) as f64 / 1_000_000.0
    }

    pub fn get_size(&self) -> f64 {
        self.size.load(Ordering::Relaxed) as f64 / 1_000_000.0
    }

    pub fn update_size(&self, new_size: f64) {
        self.size.store((new_size * 1_000_000.0) as u64, Ordering::Relaxed);
        self.timestamp.store(Utc::now().timestamp_nanos_opt().unwrap_or(0) as u64, Ordering::Relaxed);
    }

    pub fn add_size(&self, additional_size: f64) -> f64 {
        let additional_atomic = (additional_size * 1_000_000.0) as u64;
        let new_size = self.size.fetch_add(additional_atomic, Ordering::Relaxed) + additional_atomic;
        self.timestamp.store(Utc::now().timestamp_nanos_opt().unwrap_or(0) as u64, Ordering::Relaxed);
        new_size as f64 / 1_000_000.0
    }

    pub fn subtract_size(&self, size_to_remove: f64) -> f64 {
        let remove_atomic = (size_to_remove * 1_000_000.0) as u64;
        let old_size = self.size.fetch_sub(remove_atomic, Ordering::Relaxed);
        self.timestamp.store(Utc::now().timestamp_nanos_opt().unwrap_or(0) as u64, Ordering::Relaxed);
        (old_size.saturating_sub(remove_atomic)) as f64 / 1_000_000.0
    }
}

/// Lock-free circular buffer for high-frequency data
pub struct LockFreeCircularBuffer<T> {
    buffer: Vec<AtomicPtr<T>>,
    capacity: usize,
    head: AtomicUsize,
    tail: AtomicUsize,
    size: AtomicUsize,
}

impl<T> LockFreeCircularBuffer<T> {
    pub fn new(capacity: usize) -> Self {
        let mut buffer = Vec::with_capacity(capacity);
        for _ in 0..capacity {
            buffer.push(AtomicPtr::new(ptr::null_mut()));
        }

        Self {
            buffer,
            capacity,
            head: AtomicUsize::new(0),
            tail: AtomicUsize::new(0),
            size: AtomicUsize::new(0),
        }
    }

    pub fn push(&self, item: Box<T>) -> bool {
        let current_size = self.size.load(Ordering::Relaxed);
        if current_size >= self.capacity {
            return false; // Buffer full
        }

        let tail = self.tail.fetch_add(1, Ordering::Relaxed) % self.capacity;
        let item_ptr = Box::into_raw(item);
        
        // Try to store the item
        let old_ptr = self.buffer[tail].swap(item_ptr, Ordering::Relaxed);
        if !old_ptr.is_null() {
            // Slot was occupied, restore the item and fail
            let _ = unsafe { Box::from_raw(item_ptr) };
            return false;
        }

        self.size.fetch_add(1, Ordering::Relaxed);
        true
    }

    pub fn pop(&self) -> Option<Box<T>> {
        let current_size = self.size.load(Ordering::Relaxed);
        if current_size == 0 {
            return None;
        }

        let head = self.head.fetch_add(1, Ordering::Relaxed) % self.capacity;
        let item_ptr = self.buffer[head].swap(ptr::null_mut(), Ordering::Relaxed);
        
        if item_ptr.is_null() {
            return None;
        }

        self.size.fetch_sub(1, Ordering::Relaxed);
        Some(unsafe { Box::from_raw(item_ptr) })
    }

    pub fn len(&self) -> usize {
        self.size.load(Ordering::Relaxed)
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }
}

unsafe impl<T: Send> Send for LockFreeCircularBuffer<T> {}
unsafe impl<T: Send> Sync for LockFreeCircularBuffer<T> {}

/// Lock-free memory pool for order objects
pub struct LockFreeMemoryPool<T> {
    available_items: LockFreeCircularBuffer<T>,
    total_allocations: AtomicU64,
    total_deallocations: AtomicU64,
    pool_hits: AtomicU64,
    pool_misses: AtomicU64,
}

impl<T: Default> LockFreeMemoryPool<T> {
    pub fn new(initial_capacity: usize) -> Self {
        let pool = Self {
            available_items: LockFreeCircularBuffer::new(initial_capacity),
            total_allocations: AtomicU64::new(0),
            total_deallocations: AtomicU64::new(0),
            pool_hits: AtomicU64::new(0),
            pool_misses: AtomicU64::new(0),
        };

        // Pre-populate the pool
        for _ in 0..initial_capacity {
            let item = Box::new(T::default());
            pool.available_items.push(item);
        }

        pool
    }

    pub fn acquire(&self) -> Box<T> {
        self.total_allocations.fetch_add(1, Ordering::Relaxed);
        
        if let Some(item) = self.available_items.pop() {
            self.pool_hits.fetch_add(1, Ordering::Relaxed);
            item
        } else {
            self.pool_misses.fetch_add(1, Ordering::Relaxed);
            Box::new(T::default())
        }
    }

    pub fn release(&self, item: Box<T>) {
        self.total_deallocations.fetch_add(1, Ordering::Relaxed);
        
        if !self.available_items.push(item) {
            // Pool is full, just drop the item
            debug!("Memory pool full, dropping item");
        }
    }

    pub fn get_stats(&self) -> MemoryPoolStats {
        MemoryPoolStats {
            total_allocations: self.total_allocations.load(Ordering::Relaxed),
            total_deallocations: self.total_deallocations.load(Ordering::Relaxed),
            pool_hits: self.pool_hits.load(Ordering::Relaxed),
            pool_misses: self.pool_misses.load(Ordering::Relaxed),
            current_pool_size: self.available_items.len(),
            pool_capacity: self.available_items.capacity(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryPoolStats {
    pub total_allocations: u64,
    pub total_deallocations: u64,
    pub pool_hits: u64,
    pub pool_misses: u64,
    pub current_pool_size: usize,
    pub pool_capacity: usize,
}

/// Lock-free order queue using atomic operations
pub struct LockFreeOrderQueue {
    orders: LockFreeCircularBuffer<OrderRequest>,
    processing_metrics: Arc<LockFreeProcessingMetrics>,
}

impl LockFreeOrderQueue {
    pub fn new(capacity: usize) -> Self {
        Self {
            orders: LockFreeCircularBuffer::new(capacity),
            processing_metrics: Arc::new(LockFreeProcessingMetrics::new()),
        }
    }

    pub fn enqueue(&self, order: OrderRequest) -> bool {
        let success = self.orders.push(Box::new(order));
        if success {
            self.processing_metrics.orders_enqueued.fetch_add(1, Ordering::Relaxed);
        } else {
            self.processing_metrics.queue_full_events.fetch_add(1, Ordering::Relaxed);
        }
        success
    }

    pub fn dequeue(&self) -> Option<OrderRequest> {
        if let Some(order_box) = self.orders.pop() {
            self.processing_metrics.orders_dequeued.fetch_add(1, Ordering::Relaxed);
            Some(*order_box)
        } else {
            None
        }
    }

    pub fn len(&self) -> usize {
        self.orders.len()
    }

    pub fn is_empty(&self) -> bool {
        self.orders.is_empty()
    }

    pub fn get_metrics(&self) -> LockFreeQueueStats {
        LockFreeQueueStats {
            current_queue_depth: self.len(),
            queue_capacity: self.orders.capacity(),
            orders_enqueued: self.processing_metrics.orders_enqueued.load(Ordering::Relaxed),
            orders_dequeued: self.processing_metrics.orders_dequeued.load(Ordering::Relaxed),
            queue_full_events: self.processing_metrics.queue_full_events.load(Ordering::Relaxed),
        }
    }
}

/// Lock-free processing metrics
pub struct LockFreeProcessingMetrics {
    pub orders_enqueued: AtomicU64,
    pub orders_dequeued: AtomicU64,
    pub queue_full_events: AtomicU64,
    pub processing_time_total_ns: AtomicU64,
    pub successful_operations: AtomicU64,
    pub failed_operations: AtomicU64,
}

impl LockFreeProcessingMetrics {
    pub fn new() -> Self {
        Self {
            orders_enqueued: AtomicU64::new(0),
            orders_dequeued: AtomicU64::new(0),
            queue_full_events: AtomicU64::new(0),
            processing_time_total_ns: AtomicU64::new(0),
            successful_operations: AtomicU64::new(0),
            failed_operations: AtomicU64::new(0),
        }
    }

    pub fn record_operation(&self, processing_time_ns: u64, success: bool) {
        self.processing_time_total_ns.fetch_add(processing_time_ns, Ordering::Relaxed);
        if success {
            self.successful_operations.fetch_add(1, Ordering::Relaxed);
        } else {
            self.failed_operations.fetch_add(1, Ordering::Relaxed);
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockFreeQueueStats {
    pub current_queue_depth: usize,
    pub queue_capacity: usize,
    pub orders_enqueued: u64,
    pub orders_dequeued: u64,
    pub queue_full_events: u64,
}

/// Lock-free price level for order book
pub struct LockFreePriceLevel {
    price: AtomicU64, // Fixed-point representation
    total_size: AtomicU64,
    order_count: AtomicUsize,
    last_update: AtomicU64,
}

impl LockFreePriceLevel {
    pub fn new(price: f64) -> Self {
        Self {
            price: AtomicU64::new((price * 1_000_000.0) as u64),
            total_size: AtomicU64::new(0),
            order_count: AtomicUsize::new(0),
            last_update: AtomicU64::new(Utc::now().timestamp_nanos_opt().unwrap_or(0) as u64),
        }
    }

    pub fn add_order(&self, size: f64) {
        let size_atomic = (size * 1_000_000.0) as u64;
        self.total_size.fetch_add(size_atomic, Ordering::Relaxed);
        self.order_count.fetch_add(1, Ordering::Relaxed);
        self.last_update.store(Utc::now().timestamp_nanos_opt().unwrap_or(0) as u64, Ordering::Relaxed);
    }

    pub fn remove_order(&self, size: f64) -> bool {
        let size_atomic = (size * 1_000_000.0) as u64;
        let old_size = self.total_size.fetch_sub(size_atomic, Ordering::Relaxed);
        
        if old_size >= size_atomic {
            self.order_count.fetch_sub(1, Ordering::Relaxed);
            self.last_update.store(Utc::now().timestamp_nanos_opt().unwrap_or(0) as u64, Ordering::Relaxed);
            true
        } else {
            // Restore the size if subtraction would underflow
            self.total_size.fetch_add(size_atomic, Ordering::Relaxed);
            false
        }
    }

    pub fn get_price(&self) -> f64 {
        self.price.load(Ordering::Relaxed) as f64 / 1_000_000.0
    }

    pub fn get_total_size(&self) -> f64 {
        self.total_size.load(Ordering::Relaxed) as f64 / 1_000_000.0
    }

    pub fn get_order_count(&self) -> usize {
        self.order_count.load(Ordering::Relaxed)
    }

    pub fn is_empty(&self) -> bool {
        self.total_size.load(Ordering::Relaxed) == 0
    }
}
