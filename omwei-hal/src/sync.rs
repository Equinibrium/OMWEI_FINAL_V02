//! Synchronization primitives for OMWEI Equinibrium SoC
//! 
//! This module provides safe synchronization primitives for multi-core
//! systems, specifically designed for the 128-bit memory port architecture.

use core::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use core::cell::UnsafeCell;
use core::ops::{Deref, DerefMut};

/// A simple spinlock implementation for protecting shared data
/// 
/// This spinlock uses atomic operations with proper memory ordering
/// to ensure thread safety across the 4 harts in the OMWEI SoC.
/// 
/// # Type Parameters
/// * `T` - The type of data to protect
/// 
/// # Safety
/// This implementation uses Acquire/Release ordering which provides
/// the necessary memory barriers for the 128-bit memory port.
pub struct Spinlock<T> {
    locked: AtomicBool,
    data: UnsafeCell<T>,
}

/// A guard that provides access to the protected data
/// 
/// The lock is automatically released when this guard is dropped.
pub struct SpinlockGuard<'a, T> {
    lock: &'a Spinlock<T>,
}

impl<T> Spinlock<T> {
    /// Create a new spinlock with the given initial data
    pub const fn new(data: T) -> Self {
        Self {
            locked: AtomicBool::new(false),
            data: UnsafeCell::new(data),
        }
    }

    /// Acquire the lock, blocking until it becomes available
    /// 
    /// This function will spin until the lock can be acquired.
    /// Uses Acquire ordering to ensure all subsequent operations
    /// happen after acquiring the lock.
    pub fn lock(&self) -> SpinlockGuard<'_, T> {
        // Spin until we can acquire the lock
        while self.locked.compare_exchange_weak(
            false, 
            true, 
            Ordering::Acquire, 
            Ordering::Relaxed
        ).is_err() {
            // Add memory barrier for 128-bit port consistency
            unsafe {
                core::arch::asm!("fence iorw, iorw");
            }
            
            // Use WFI to save power while waiting (optional)
            unsafe {
                core::arch::asm!("wfi");
            }
        }
        
        SpinlockGuard { lock: self }
    }

    /// Try to acquire the lock without blocking
    /// 
    /// Returns None if the lock is already taken, otherwise returns
    /// a guard that provides access to the protected data.
    pub fn try_lock(&self) -> Option<SpinlockGuard<'_, T>> {
        if self.locked.compare_exchange(
            false, 
            true, 
            Ordering::Acquire, 
            Ordering::Relaxed
        ).is_ok() {
            Some(SpinlockGuard { lock: self })
        } else {
            None
        }
    }

    /// Force unlock the lock (unsafe)
    /// 
    /// # Safety
    /// This function is unsafe because it can lead to data races
    /// if used incorrectly. Only use this if you're absolutely sure
    /// the lock is not currently held.
    pub unsafe fn force_unlock(&self) {
        self.locked.store(false, Ordering::Release);
        core::arch::asm!("fence iorw, iorw");
    }
}

impl<'a, T> Drop for SpinlockGuard<'a, T> {
    fn drop(&mut self) {
        // Release the lock with Release ordering
        self.lock.locked.store(false, Ordering::Release);
        
        // Memory barrier to ensure the unlock is visible to all harts
        unsafe {
            core::arch::asm!("fence iorw, iorw");
        }
    }
}

impl<'a, T> Deref for SpinlockGuard<'a, T> {
    type Target = T;
    
    fn deref(&self) -> &Self::Target {
        unsafe { &*self.lock.data.get() }
    }
}

impl<'a, T> DerefMut for SpinlockGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.lock.data.get() }
    }
}

/// Thread-safe atomic counter for multi-core operations
/// 
/// This provides a simple way to safely increment/decrement counters
/// across multiple harts without requiring a lock.
pub struct AtomicCounter {
    value: AtomicU64,
}

impl AtomicCounter {
    /// Create a new atomic counter with initial value 0
    pub const fn new() -> Self {
        Self {
            value: AtomicU64::new(0),
        }
    }

    /// Create a new atomic counter with the given initial value
    pub const fn with_value(value: u64) -> Self {
        Self {
            value: AtomicU64::new(value),
        }
    }

    /// Increment the counter and return the new value
    /// 
    /// Uses Acquire/Release ordering for proper memory consistency
    /// across the 128-bit memory port.
    pub fn increment(&self) -> u64 {
        let result = self.value.fetch_add(1, Ordering::AcqRel);
        // Memory barrier for consistency
        unsafe {
            core::arch::asm!("fence iorw, iorw");
        }
        result + 1
    }

    /// Decrement the counter and return the new value
    pub fn decrement(&self) -> u64 {
        let result = self.value.fetch_sub(1, Ordering::AcqRel);
        // Memory barrier for consistency
        unsafe {
            core::arch::asm!("fence iorw, iorw");
        }
        result - 1
    }

    /// Get the current value
    pub fn get(&self) -> u64 {
        let result = self.value.load(Ordering::Acquire);
        // Memory barrier for consistency
        unsafe {
            core::arch::asm!("fence iorw, iorw");
        }
        result
    }

    /// Set the counter to a specific value
    pub fn set(&self, value: u64) {
        self.value.store(value, Ordering::Release);
        // Memory barrier for consistency
        unsafe {
            core::arch::asm!("fence iorw, iorw");
        }
    }

    /// Add a value to the counter and return the new value
    pub fn add(&self, delta: u64) -> u64 {
        let result = self.value.fetch_add(delta, Ordering::AcqRel);
        // Memory barrier for consistency
        unsafe {
            core::arch::asm!("fence iorw, iorw");
        }
        result + delta
    }

    /// Subtract a value from the counter and return the new value
    pub fn sub(&self, delta: u64) -> u64 {
        let result = self.value.fetch_sub(delta, Ordering::AcqRel);
        // Memory barrier for consistency
        unsafe {
            core::arch::asm!("fence iorw, iorw");
        }
        result - delta
    }
}

// Safety: Spinlock and SpinlockGuard are Send and Sync because
// the atomic operations ensure thread safety
unsafe impl<T: Send> Send for Spinlock<T> {}
unsafe impl<T: Send> Sync for Spinlock<T> {}

impl<T> From<T> for Spinlock<T> {
    fn from(data: T) -> Self {
        Spinlock::new(data)
    }
}
