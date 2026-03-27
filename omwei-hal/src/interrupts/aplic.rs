//! Advanced Platform-Level Interrupt Controller (APLIC) driver for OMWEI Equinibrium SoC
//! 
//! This module provides a safe interface to the APLIC interrupt controller,
//! which supports 511 interrupt sources with 31 priority levels and 4 harts.

use crate::registers::aplic;
use crate::registers::aplic::constants::*;
use riscv::register::mhartid;

/// APLIC driver instance
/// 
/// This struct provides access to the Advanced Platform-Level Interrupt Controller.
/// It handles interrupt routing, priority management, and claim/complete operations.
pub struct Aplic {
    initialized: bool,
}

/// Interrupt source configuration
#[derive(Debug, Clone, Copy)]
pub struct SourceConfig {
    pub delivery_mode: u32,
    pub target_mode: u32,
    pub priority: u32,
}

impl Default for SourceConfig {
    fn default() -> Self {
        Self {
            delivery_mode: delivery::INACTIVE,
            target_mode: target_mode::SPECIFIC_HART,
            priority: 0,
        }
    }
}

/// Interrupt target configuration
#[derive(Debug, Clone, Copy)]
pub struct TargetConfig {
    pub hart_id: usize,
    pub priority: u32,
    pub enabled: bool,
}

impl Default for TargetConfig {
    fn default() -> Self {
        Self {
            hart_id: 0,
            priority: 0,
            enabled: false,
        }
    }
}

impl Aplic {
    /// Create a new APLIC instance
    pub const fn new() -> Self {
        Self {
            initialized: false,
        }
    }

    /// Initialize the APLIC
    /// 
    /// This function configures the APLIC domain to little-endian mode
    /// and enables the domain for operation.
    /// 
    /// # Safety
    /// This function should only be called once during system initialization.
    pub fn init(&mut self) {
        if self.initialized {
            return;
        }

        unsafe {
            // Set domain configuration: enable domain, little-endian mode
            let domaincfg_ptr = aplic::DOMAINCFG as *mut u32;
            let config = 0b01; // Bit 0: enable, Bit 1: little-endian
            domaincfg_ptr.write_volatile(config);

            // Memory barrier to ensure the configuration is visible
            core::arch::asm!("fence iorw, iorw");

            // Initialize all interrupt sources to inactive
            for source_id in 1..=NUM_SOURCES {
                let sourcecfg_ptr = aplic::sourcecfg(source_id) as *mut u32;
                sourcecfg_ptr.write_volatile(delivery::INACTIVE);

                // Initialize target registers
                let target_ptr = aplic::target(source_id) as *mut u32;
                target_ptr.write_volatile(0); // Disabled
            }

            // Initialize hart-specific registers
            for hart_id in 0..NUM_HARTS {
                // Enable interrupt delivery for all harts
                let idelivery_ptr = aplic::idelivery(hart_id) as *mut u32;
                idelivery_ptr.write_volatile(1);

                // Set threshold to 0 (allow all interrupts)
                let ithreshold_ptr = aplic::ithreshold(hart_id) as *mut u32;
                ithreshold_ptr.write_volatile(0);
            }

            // Memory barrier to ensure all initialization is complete
            core::arch::asm!("fence iorw, iorw");
        }

        self.initialized = true;
    }

    /// Enable an interrupt source
    /// 
    /// Configures an interrupt source with the specified delivery mode,
    /// priority, and target hart.
    /// 
    /// # Arguments
    /// * `source_id` - Interrupt source ID (1-511)
    /// * `config` - Source configuration (delivery mode, target mode, priority)
    /// * `target` - Target configuration (hart ID, priority, enabled)
    /// 
    /// # Panics
    /// Panics if source_id is out of range or priority is invalid.
    pub fn enable_source(&self, source_id: usize, config: SourceConfig, target: TargetConfig) {
        if !self.initialized {
            panic!("APLIC not initialized");
        }

        if source_id == 0 || source_id > NUM_SOURCES {
            panic!("Invalid source ID: {}", source_id);
        }

        if config.priority > MAX_PRIORITY {
            panic!("Invalid priority: {}", config.priority);
        }

        if target.hart_id >= NUM_HARTS {
            panic!("Invalid hart ID: {}", target.hart_id);
        }

        unsafe {
            // Configure source
            let sourcecfg_value = (config.delivery_mode & 0xF) |
                                ((config.target_mode & 0xF) << 4) |
                                ((config.priority & 0xFF) << 8);
            
            let sourcecfg_ptr = aplic::sourcecfg(source_id) as *mut u32;
            sourcecfg_ptr.write_volatile(sourcecfg_value);

            // Configure target
            let target_value = (target.hart_id as u32 & 0x3) |
                             ((target.priority & 0x1F) << 2) |
                             ((target.enabled as u32) << 7);
            
            let target_ptr = aplic::target(source_id) as *mut u32;
            target_ptr.write_volatile(target_value);

            // Memory barrier to ensure configuration is visible
            core::arch::asm!("fence iorw, iorw");
        }
    }

    /// Disable an interrupt source
    /// 
    /// # Arguments
    /// * `source_id` - Interrupt source ID (1-511)
    pub fn disable_source(&self, source_id: usize) {
        if !self.initialized {
            panic!("APLIC not initialized");
        }

        if source_id == 0 || source_id > NUM_SOURCES {
            panic!("Invalid source ID: {}", source_id);
        }

        unsafe {
            // Set source to inactive
            let sourcecfg_ptr = aplic::sourcecfg(source_id) as *mut u32;
            sourcecfg_ptr.write_volatile(delivery::INACTIVE);

            // Disable target
            let target_ptr = aplic::target(source_id) as *mut u32;
            target_ptr.write_volatile(0);

            // Memory barrier to ensure changes are visible
            core::arch::asm!("fence iorw, iorw");
        }
    }

    /// Set interrupt priority threshold for a hart
    /// 
    /// Only interrupts with priority >= threshold will be delivered to the hart.
    /// 
    /// # Arguments
    /// * `hart_id` - Hart ID (0-3)
    /// * `threshold` - Priority threshold (0-31)
    pub fn set_threshold(&self, hart_id: usize, threshold: u32) {
        if !self.initialized {
            panic!("APLIC not initialized");
        }

        if hart_id >= NUM_HARTS {
            panic!("Invalid hart ID: {}", hart_id);
        }

        if threshold > MAX_PRIORITY {
            panic!("Invalid threshold: {}", threshold);
        }

        unsafe {
            let ithreshold_ptr = aplic::ithreshold(hart_id) as *mut u32;
            ithreshold_ptr.write_volatile(threshold & 0x1F);

            // Memory barrier to ensure threshold is set
            core::arch::asm!("fence iorw, iorw");
        }
    }

    /// Enable/disable interrupt delivery for a hart
    /// 
    /// # Arguments
    /// * `hart_id` - Hart ID (0-3)
    /// * `enable` - true to enable, false to disable
    pub fn set_delivery_enable(&self, hart_id: usize, enable: bool) {
        if !self.initialized {
            panic!("APLIC not initialized");
        }

        if hart_id >= NUM_HARTS {
            panic!("Invalid hart ID: {}", hart_id);
        }

        unsafe {
            let idelivery_ptr = aplic::idelivery(hart_id) as *mut u32;
            idelivery_ptr.write_volatile(if enable { 1 } else { 0 });

            // Memory barrier to ensure setting is applied
            core::arch::asm!("fence iorw, iorw");
        }
    }

    /// Claim an interrupt for the current hart
    /// 
    /// This function claims the highest priority pending interrupt
    /// for the current hart and returns the interrupt source ID.
    /// 
    /// # Returns
    /// The interrupt source ID, or 0 if no interrupt is pending
    pub fn claim_interrupt(&self) -> usize {
        if !self.initialized {
            panic!("APLIC not initialized");
        }

        let hart_id = mhartid::read();
        if hart_id >= NUM_HARTS {
            panic!("Invalid hart ID: {}", hart_id);
        }

        unsafe {
            let claim_ptr = aplic::claim(hart_id) as *mut u32;
            let interrupt_id = claim_ptr.read_volatile();

            // Memory barrier to ensure the claim is processed
            core::arch::asm!("fence iorw, iorw");

            interrupt_id as usize
        }
    }

    /// Complete an interrupt
    /// 
    /// This function signals that an interrupt has been processed.
    /// 
    /// # Arguments
    /// * `interrupt_id` - The interrupt source ID to complete
    pub fn complete_interrupt(&self, interrupt_id: usize) {
        if !self.initialized {
            panic!("APLIC not initialized");
        }

        if interrupt_id == 0 || interrupt_id > NUM_SOURCES {
            panic!("Invalid interrupt ID: {}", interrupt_id);
        }

        let hart_id = mhartid::read();
        if hart_id >= NUM_HARTS {
            panic!("Invalid hart ID: {}", hart_id);
        }

        unsafe {
            let complete_ptr = aplic::complete(hart_id) as *mut u32;
            complete_ptr.write_volatile(interrupt_id as u32);

            // Memory barrier to ensure completion is processed
            core::arch::asm!("fence iorw, iorw");
        }
    }

    /// Get the highest priority pending interrupt for a hart
    /// 
    /// This function returns the highest priority pending interrupt
    /// without claiming it.
    /// 
    /// # Arguments
    /// * `hart_id` - Hart ID (0-3)
    /// 
    /// # Returns
    /// The interrupt source ID, or 0 if no interrupt is pending
    pub fn get_pending_interrupt(&self, hart_id: usize) -> usize {
        if !self.initialized {
            panic!("APLIC not initialized");
        }

        if hart_id >= NUM_HARTS {
            panic!("Invalid hart ID: {}", hart_id);
        }

        unsafe {
            let identity_ptr = aplic::identity(hart_id) as *const u32;
            let interrupt_id = identity_ptr.read_volatile();

            // Memory barrier to ensure read is complete
            core::arch::asm!("fence iorw, iorw");

            interrupt_id as usize
        }
    }

    /// Check if the APLIC is initialized
    pub fn is_initialized(&self) -> bool {
        self.initialized
    }

    /// Get the current domain configuration
    pub fn get_domain_config(&self) -> u32 {
        if !self.initialized {
            return 0;
        }

        unsafe {
            let domaincfg_ptr = aplic::DOMAINCFG as *const u32;
            let config = domaincfg_ptr.read_volatile();

            // Memory barrier to ensure read is complete
            core::arch::asm!("fence iorw, iorw");

            config
        }
    }
}

/// Global APLIC instance
/// 
/// This is the single APLIC instance that should be used throughout
/// the system.
pub static mut APLIC: Aplic = Aplic::new();

// Safety: Aplic is Send but not Sync - access should be synchronized
unsafe impl Send for Aplic {}

impl Default for Aplic {
    fn default() -> Self {
        Self::new()
    }
}
