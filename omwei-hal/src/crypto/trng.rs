//! True Random Number Generator (TRNG) driver for OMWEI Equinibrium SoC
//! 
//! This module provides a safe interface to the Hardware Crypto Accelerator's
//! TRNG peripheral, which generates high-quality random numbers suitable
//! for cryptographic purposes.

use crate::registers::hca::trng;
use core::sync::atomic::{AtomicBool, Ordering};

/// TRNG driver instance
/// 
/// This struct provides access to the True Random Number Generator
/// in the Hardware Crypto Accelerator. Only one instance should exist
/// at a time to prevent conflicts between cores.
pub struct Trng {
    initialized: AtomicBool,
}

/// TRNG Control Register bits
mod ctrl_bits {
    pub const ENABLE: u32 = 0x01;
    pub const START_OSC: u32 = 0x02;
}

/// TRNG Status Register bits
mod status_bits {
    pub const DATA_READY: u32 = 0x01;
    pub const OSC_STABLE: u32 = 0x02;
}

impl Trng {
    /// Create a new TRNG instance
    /// 
    /// The TRNG is not initialized until `init()` is called.
    pub const fn new() -> Self {
        Self {
            initialized: AtomicBool::new(false),
        }
    }

    /// Initialize the TRNG peripheral
    /// 
    /// This function enables the TRNG clock and starts the oscillator.
    /// It waits for the oscillator to stabilize before returning.
    /// 
    /// # Safety
    /// This function should only be called once and should be protected
    /// by a mutex or spinlock in multi-core environments.
    pub fn init(&self) {
        if self.initialized.load(Ordering::Acquire) {
            return; // Already initialized
        }

        // Set initialized flag
        self.initialized.store(true, Ordering::SeqCst);
        
        unsafe {
            let ctrl_ptr = trng::CTRL as *mut u32;
            
            // Enable TRNG and start oscillator
            ctrl_ptr.write_volatile(ctrl_bits::ENABLE | ctrl_bits::START_OSC);
            
            // Memory barrier to ensure initialization is visible
            core::arch::asm!("fence iorw, iorw");
            
            // In QEMU simulation, the virtual oscillator never "warms up"
            // Add a timeout to bypass the wait in simulation mode
            let mut timeout = 1000000; // Large timeout for simulation
            while timeout > 0 {
                let status_ptr = trng::STATUS as *const u32;
                let status = status_ptr.read_volatile();
                
                if (status & status_bits::OSC_STABLE) != 0 {
                    break; // Oscillator is stable
                }
                for _ in 0..100 {
                    core::arch::asm!("nop");
                }
                timeout -= 1;
            }

            if timeout == 0 {
                panic!("TRNG oscillator failed to stabilize");
            }

            // Memory barrier to ensure status is read correctly
            core::arch::asm!("fence iorw, iorw");
        }

        self.initialized.store(true, Ordering::Release);
    }

    /// Get a 32-bit random number
    /// 
    /// This function polls the TRNG status register until data is available,
    /// then reads a 32-bit random number from the FIFO.
    /// 
    /// # Panics
    /// Panics if the TRNG is not initialized or if a timeout occurs.
    /// 
    /// # Returns
    /// A cryptographically secure 32-bit random number
    pub fn get_u32(&self) -> u32 {
        if !self.initialized.load(Ordering::Acquire) {
            panic!("TRNG not initialized");
        }

        unsafe {
            // Wait for data to be available
            let mut timeout = 1000000; // Timeout counter
            while timeout > 0 {
                let status_ptr = trng::STATUS as *const u32;
                let status = status_ptr.read_volatile();
                
                if (status & status_bits::DATA_READY) != 0 {
                    break;
                }
                
                // Small delay
                for _ in 0..100 {
                    core::arch::asm!("nop");
                }
                timeout -= 1;
            }

            if timeout == 0 {
                panic!("TRNG data not available");
            }

            // Read random number from FIFO
            let fifo_ptr = trng::FIFO as *const u32;
            let random_value = fifo_ptr.read_volatile();

            // Memory barrier to ensure the value is synchronized
            core::arch::asm!("fence iorw, iorw");

            random_value
        }
    }

    /// Fill a buffer with random bytes
    /// 
    /// This function fills the provided buffer with cryptographically
    /// secure random bytes.
    /// 
    /// # Arguments
    /// * `buffer` - Mutable slice to fill with random data
    /// 
    /// # Panics
    /// Panics if the TRNG is not initialized
    pub fn fill_bytes(&self, buffer: &mut [u8]) {
        if !self.initialized.load(Ordering::Acquire) {
            panic!("TRNG not initialized");
        }

        // Fill buffer 4 bytes at a time
        let mut chunks = buffer.chunks_exact_mut(4);
        for chunk in &mut chunks {
            let random = self.get_u32();
            chunk.copy_from_slice(&random.to_le_bytes());
        }

        // Handle remaining bytes (if any)
        let remainder = chunks.into_remainder();
        if !remainder.is_empty() {
            let random = self.get_u32();
            let random_bytes = random.to_le_bytes();
            remainder.copy_from_slice(&random_bytes[..remainder.len()]);
        }
    }

    /// Check if the TRNG is initialized
    pub fn is_initialized(&self) -> bool {
        self.initialized.load(Ordering::Acquire)
    }

    /// Get the current TRNG status
    /// 
    /// Returns the raw status register value for debugging purposes.
    pub fn get_status(&self) -> u32 {
        if !self.initialized.load(Ordering::Acquire) {
            return 0;
        }

        unsafe {
            let status_ptr = trng::STATUS as *const u32;
            let status = status_ptr.read_volatile();
            
            // Memory barrier to ensure status is read correctly
            core::arch::asm!("fence iorw, iorw");
            
            status
        }
    }
}

/// Global TRNG instance
/// 
/// This is the single TRNG instance that should be used throughout
/// the system. Access should be synchronized between cores.
pub static TRNG: Trng = Trng::new();

// Safety: Trng is Send but not Sync - only one core should access it at a time
unsafe impl Send for Trng {}

impl Default for Trng {
    fn default() -> Self {
        Self::new()
    }
}
