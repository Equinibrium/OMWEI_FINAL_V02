//! OMWEI Equinibrium SoC Register Definitions
//! 
//! This module defines the base addresses and register offsets for
//! the various peripherals in the OMWEI Equinibrium SoC.

use core::ptr::{read_volatile, write_volatile};

/// Simulation mode flag for QEMU testing
/// When true, SLC hardware operations are mocked to avoid hanging
pub const IS_SIMULATION: bool = true;

/// CLINT (Core Local Interruptor) Base Address
/// Timer and Inter-Processor Interrupt controller
pub const CLINT_BASE: usize = 0x0200_0000;

/// CLINT Register Offsets
pub mod clint {
    use super::CLINT_BASE;
    
    /// MSIP (Machine Software Interrupt Pending) registers
    /// One per hart, 4 bytes each, starting at CLINT_BASE
    pub const MSIP_BASE: usize = CLINT_BASE + 0x0000;
    
    /// Get MSIP register address for a specific hart
    pub const fn msip(hart_id: usize) -> usize {
        MSIP_BASE + hart_id * 4
    }
    
    /// MTIMECMP (Machine Timer Compare) registers
    /// One per hart, 8 bytes each
    pub const MTIMECMP: usize = CLINT_BASE + 0x4000;
    
    /// MTIME (Machine Timer) register
    /// 64-bit counter shared by all harts
    pub const MTIME: usize = CLINT_BASE + 0xBFF8;
}

/// APLIC (Advanced Platform-Level Interrupt Controller) Base Address
/// Advanced interrupt controller with 511 inputs and 31 priorities
pub const APLIC_BASE: usize = 0x0C00_0000;

/// UART0 Base Address (NS16550A UART for QEMU virt machine)
/// UART0 serial console for debug output and logging
pub const UART0_BASE: usize = 0x10000000;

/// APLIC Register Offsets
pub mod aplic {
    use super::APLIC_BASE;
    
    /// Domain Configuration Register
    /// Bit 0: Enable domain (1=enabled, 0=disabled)
    /// Bit 1: Endianness (0=little-endian, 1=big-endian)
    pub const DOMAINCFG: usize = APLIC_BASE + 0x0000;
    
    /// Source Configuration Registers
    /// One per interrupt source, 4 bytes each
    /// Bits 0-3: Delivery mode (0=inactive, 1=edge-rising, 4=level-high)
    /// Bits 4-7: Target mode (0=specific hart, 1=all harts)
    /// Bits 8-15: Priority (0-31)
    pub const SRCCFG: usize = APLIC_BASE + 0x0004;
    
    /// Get source configuration register address for a specific source
    pub const fn sourcecfg(source_id: usize) -> usize {
        SRCCFG + source_id * 4
    }
    
    /// Interrupt Enable Registers
    /// 32 bits per register, covering 511 sources
    pub const IEN: usize = APLIC_BASE + 0x2000;
    
    /// Set Interrupt Enable Registers
    pub const IENS: usize = APLIC_BASE + 0x2080;
    
    /// Clear Interrupt Enable Registers
    pub const IENC: usize = APLIC_BASE + 0x2100;
    
    /// Set Pending Registers
    pub const IP: usize = APLIC_BASE + 0x3000;
    
    /// Target Registers
    /// One per interrupt source, 4 bytes each
    /// Bits 0-1: Target hart ID (0-3)
    /// Bits 2-6: Priority (0-31)
    /// Bit 7: Enable (1=enabled, 0=disabled)
    pub const TARGET: usize = APLIC_BASE + 0x4000;
    
    /// Get target register address for a specific source
    pub const fn target(source_id: usize) -> usize {
        TARGET + source_id * 4
    }
    
    /// Identity Registers
    /// One per hart, 4 bytes each
    /// Returns the highest priority pending interrupt for the hart
    pub const IDENTITY: usize = APLIC_BASE + 0x5000;
    
    /// Get identity register address for a specific hart
    pub const fn identity(hart_id: usize) -> usize {
        IDENTITY + hart_id * 4
    }
    
    /// Claim/Complete Registers
    /// One per hart, 4 bytes each
    /// Read to claim an interrupt, write to complete it
    pub const CLAIM: usize = APLIC_BASE + 0x6000;
    
    /// Get claim register address for a specific hart
    pub const fn claim(hart_id: usize) -> usize {
        CLAIM + hart_id * 4
    }
    
    /// Complete Registers
    /// One per hart, 4 bytes each
    /// Write to complete a claimed interrupt
    pub const COMPLETE: usize = APLIC_BASE + 0x6040;
    
    /// Get complete register address for a specific hart
    pub const fn complete(hart_id: usize) -> usize {
        COMPLETE + hart_id * 4
    }
    
    /// Interrupt Delivery Enable Registers
    /// One per hart, 4 bytes each
    /// Bit 0: Enable interrupt delivery (1=enabled, 0=disabled)
    pub const IDELIVERY: usize = APLIC_BASE + 0x7000;
    
    /// Get delivery enable register address for a specific hart
    pub const fn idelivery(hart_id: usize) -> usize {
        IDELIVERY + hart_id * 4
    }
    
    /// Interrupt Threshold Registers
    /// One per hart, 4 bytes each
    /// Bits 0-4: Priority threshold (0-31)
    /// Only interrupts with priority >= threshold will be delivered
    pub const ITHRESHOLD: usize = APLIC_BASE + 0x7080;
    
    /// Get threshold register address for a specific hart
    pub const fn ithreshold(hart_id: usize) -> usize {
        ITHRESHOLD + hart_id * 4
    }
    
    /// APLIC Constants
    pub mod constants {
        /// Number of interrupt sources
        pub const NUM_SOURCES: usize = 511;
        
        /// Number of harts
        pub const NUM_HARTS: usize = 4;
        
        /// Maximum priority level
        pub const MAX_PRIORITY: u32 = 31;
        
        /// Source configuration delivery modes
        pub mod delivery {
            pub const INACTIVE: u32 = 0;
            pub const EDGE_RISING: u32 = 1;
            pub const LEVEL_HIGH: u32 = 4;
        }
        
        /// Source configuration target modes
        pub mod target_mode {
            pub const SPECIFIC_HART: u32 = 0;
            pub const ALL_HARTS: u32 = 1;
        }
    }
}

/// PMC (Power Management Controller) Base Address
pub const PMC_BASE: usize = 0x0310_0000;

/// SLC (Semantic Logic Core) Base Address
/// Mapped via CLP0 for hardware-accelerated semantic operations
pub const SLC_BASE: usize = 0x7000_0000;

/// SLC Register Offsets
pub mod slc {
    use super::SLC_BASE;
    
    /// Atom A registers (256-bit = 4 x 64-bit registers)
    /// Starting at offset 0x00
    pub mod atom_a {
        use super::SLC_BASE;
        
        /// Atom A Word 0 (bits 63:0)
        pub const WORD_0: usize = SLC_BASE + 0x00;
        
        /// Atom A Word 1 (bits 127:64)
        pub const WORD_1: usize = SLC_BASE + 0x08;
        
        /// Atom A Word 2 (bits 191:128)
        pub const WORD_2: usize = SLC_BASE + 0x10;
        
        /// Atom A Word 3 (bits 255:192)
        pub const WORD_3: usize = SLC_BASE + 0x18;
    }
    
    /// Atom B registers (256-bit = 4 x 64-bit registers)
    /// Starting at offset 0x20
    pub mod atom_b {
        use super::SLC_BASE;
        
        /// Atom B Word 0 (bits 63:0)
        pub const WORD_0: usize = SLC_BASE + 0x20;
        
        /// Atom B Word 1 (bits 127:64)
        pub const WORD_1: usize = SLC_BASE + 0x28;
        
        /// Atom B Word 2 (bits 191:128)
        pub const WORD_2: usize = SLC_BASE + 0x30;
        
        /// Atom B Word 3 (bits 255:192)
        pub const WORD_3: usize = SLC_BASE + 0x38;
    }
    
    /// Control and Status Register
    /// Offset 0x40
    /// Bit 0: Start (1 to start comparison)
    /// Bit 1: Busy (1 when operation in progress)
    /// Bit 2: Match result (1 when atoms match, 0 when they differ)
    /// Bit 3-7: Reserved
    /// Bit 8-15: Error codes
    /// Bit 16-31: Status flags
    pub const CTRL_STAT: usize = SLC_BASE + 0x40;
    
    /// Configuration Register
    /// Offset 0x44
    /// Bit 0-3: Operation mode (0=compare, 1=query, 2=link, 3=merge)
    /// Bit 4-7: Priority level
    /// Bit 8-15: Timeout configuration
    /// Bit 16-31: Reserved
    pub const CONFIG: usize = SLC_BASE + 0x44;
    
    /// Interrupt Enable Register
    /// Offset 0x48
    /// Bit 0: Enable completion interrupt
    /// Bit 1: Enable error interrupt
    /// Bit 2-31: Reserved
    pub const INT_EN: usize = SLC_BASE + 0x48;
    
    /// Interrupt Status Register
    /// Offset 0x4C
    /// Bit 0: Completion interrupt pending
    /// Bit 1: Error interrupt pending
    /// Bit 2-31: Reserved
    pub const INT_STAT: usize = SLC_BASE + 0x4C;
    
    /// Performance Counter Register
    /// Offset 0x50
    /// Counts number of operations performed
    pub const PERF_COUNT: usize = SLC_BASE + 0x50;
    
    /// Version Register
    /// Offset 0x54
    /// Bit 0-7: Minor version
    /// Bit 8-15: Major version
    /// Bit 16-31: Reserved
    pub const VERSION: usize = SLC_BASE + 0x54;
    
    /// Control and Status Register bit definitions
    pub mod ctrl_stat {
        /// Start bit - set to 1 to start operation
        pub const START: u32 = 0x01;
        
        /// Busy bit - 1 when operation is in progress
        pub const BUSY: u32 = 0x02;
        
        /// Match result bit - 1 when atoms match
        pub const MATCH: u32 = 0x04;
        
        /// Error bit - 1 when error occurred
        pub const ERROR: u32 = 0x08;
        
        /// Completion bit - 1 when operation completed
        pub const COMPLETE: u32 = 0x10;
        
        /// Timeout bit - 1 when operation timed out
        pub const TIMEOUT: u32 = 0x20;
    }
    
    /// Configuration Register bit definitions
    pub mod config {
        /// Operation modes
        pub mod mode {
            pub const COMPARE: u32 = 0x0;
            pub const QUERY: u32 = 0x1;
            pub const LINK: u32 = 0x2;
            pub const MERGE: u32 = 0x3;
        }
        
        /// Priority levels
        pub mod priority {
            pub const LOW: u32 = 0x0;
            pub const NORMAL: u32 = 0x1;
            pub const HIGH: u32 = 0x2;
            pub const CRITICAL: u32 = 0x3;
        }
    }
    
    /// Helper functions for register access
    pub fn write_atom_a(atom: &[u64; 4]) {
        unsafe {
            // Write 256-bit atom A using four 64-bit writes
            let ptr_0 = atom_a::WORD_0 as *mut u64;
            let ptr_1 = atom_a::WORD_1 as *mut u64;
            let ptr_2 = atom_a::WORD_2 as *mut u64;
            let ptr_3 = atom_a::WORD_3 as *mut u64;
            
            ptr_0.write_volatile(atom[0]);
            ptr_1.write_volatile(atom[1]);
            ptr_2.write_volatile(atom[2]);
            ptr_3.write_volatile(atom[3]);
            
            // Memory barrier to ensure all writes are visible
            core::arch::asm!("fence iorw, iorw");
        }
    }
    
    pub fn write_atom_b(atom: &[u64; 4]) {
        unsafe {
            // Write 256-bit atom B using four 64-bit writes
            let ptr_0 = atom_b::WORD_0 as *mut u64;
            let ptr_1 = atom_b::WORD_1 as *mut u64;
            let ptr_2 = atom_b::WORD_2 as *mut u64;
            let ptr_3 = atom_b::WORD_3 as *mut u64;
            
            ptr_0.write_volatile(atom[0]);
            ptr_1.write_volatile(atom[1]);
            ptr_2.write_volatile(atom[2]);
            ptr_3.write_volatile(atom[3]);
            
            // Memory barrier to ensure all writes are visible
            core::arch::asm!("fence iorw, iorw");
        }
    }
    
    pub fn start_operation() {
        unsafe {
            let ctrl_ptr = CTRL_STAT as *mut u32;
            ctrl_ptr.write_volatile(ctrl_stat::START);
            
            // Memory barrier to ensure start signal is visible
            core::arch::asm!("fence iorw, iorw");
        }
    }
    
    pub fn read_ctrl_stat() -> u32 {
        unsafe {
            let ctrl_ptr = CTRL_STAT as *const u32;
            let status = ctrl_ptr.read_volatile();
            
            // Memory barrier to ensure read is complete
            core::arch::asm!("fence iorw, iorw");
            
            status
        }
    }
    
    pub fn is_busy() -> bool {
        read_ctrl_stat() & ctrl_stat::BUSY != 0
    }
    
    pub fn get_match_result() -> bool {
        read_ctrl_stat() & ctrl_stat::MATCH != 0
    }
    
    pub fn wait_for_completion() {
        if super::IS_SIMULATION {
            // In simulation mode, just do a small delay to simulate operation
            for _ in 0..1000 {
                unsafe {
                    core::arch::asm!("nop");
                }
            }
            return;
        }
        
        // Wait for operation to complete
        while is_busy() {
            // Small delay to prevent busy-waiting
            for _ in 0..100 {
                unsafe {
                    core::arch::asm!("nop");
                }
            }
        }
    }
    
    pub fn reset() {
        unsafe {
            let ctrl_ptr = CTRL_STAT as *mut u32;
            // Clear all status bits by writing 0
            ctrl_ptr.write_volatile(0);
            
            // Memory barrier to ensure reset is visible
            core::arch::asm!("fence iorw, iorw");
        }
    }
}

/// HCA (Hardware Crypto Accelerator) Base Address
pub const HCA_BASE: usize = 0x051A_0000;

/// HCA Register Offsets
pub mod hca {
    use super::HCA_BASE;
    
    /// TRNG (True Random Number Generator) Registers
    pub mod trng {
        use super::HCA_BASE;
        
        /// TRNG Control Register
        /// Bit 0: Enable TRNG
        /// Bit 1: Start TRNG oscillator
        pub const CTRL: usize = HCA_BASE + 0x0000;
        
        /// TRNG FIFO Data Register
        /// Read 32-bit random numbers from here
        pub const FIFO: usize = HCA_BASE + 0x0004;
        
        /// TRNG Status Register
        /// Bit 0: Data ready in FIFO
        /// Bit 1: TRNG oscillator stable
        pub const STATUS: usize = HCA_BASE + 0x0008;
    }
    
    /// AES-256 Registers
    pub mod aes {
        use super::HCA_BASE;
        
        /// AES Control Register
        /// Bit 0: Enable AES engine
        /// Bit 1: Start encryption/decryption
        /// Bit 2-3: Operation mode (00=ECB, 01=CBC, 10=CTR, 11=GCM)
        /// Bit 4: Direction (0=encrypt, 1=decrypt)
        pub const CTRL: usize = HCA_BASE + 0x0100;
        
        /// AES Key Registers (256-bit = 8 x 32-bit registers)
        pub const KEY_0: usize = HCA_BASE + 0x0104;
        pub const KEY_1: usize = HCA_BASE + 0x0108;
        pub const KEY_2: usize = HCA_BASE + 0x010C;
        pub const KEY_3: usize = HCA_BASE + 0x0110;
        pub const KEY_4: usize = HCA_BASE + 0x0114;
        pub const KEY_5: usize = HCA_BASE + 0x0118;
        pub const KEY_6: usize = HCA_BASE + 0x011C;
        pub const KEY_7: usize = HCA_BASE + 0x0120;
        
        /// AES Data Input Registers (128-bit = 4 x 32-bit registers)
        pub const DIN_0: usize = HCA_BASE + 0x0124;
        pub const DIN_1: usize = HCA_BASE + 0x0128;
        pub const DIN_2: usize = HCA_BASE + 0x012C;
        pub const DIN_3: usize = HCA_BASE + 0x0130;
        
        /// AES Data Output Registers (128-bit = 4 x 32-bit registers)
        pub const DOUT_0: usize = HCA_BASE + 0x0134;
        pub const DOUT_1: usize = HCA_BASE + 0x0138;
        pub const DOUT_2: usize = HCA_BASE + 0x013C;
        pub const DOUT_3: usize = HCA_BASE + 0x0140;
        
        /// AES Initialization Vector (for CBC/CTR/GCM modes)
        pub const IV_0: usize = HCA_BASE + 0x0144;
        pub const IV_1: usize = HCA_BASE + 0x0148;
        pub const IV_2: usize = HCA_BASE + 0x014C;
        pub const IV_3: usize = HCA_BASE + 0x0150;
    }
}

/// ITIM (Instruction Tightly Integrated Memory) Base Address
pub const ITIM_BASE: usize = 0x0180_0000;

/// DLS (Data Local Storage) Base Address
pub const DLS_BASE: usize = 0x1800_0000;

/// Memory region definitions
pub mod memory {
    /// FLASH (Peripheral Port) Base Address
    pub const FLASH_BASE: usize = 0x2000_0000;
    
    /// RAM (128-bit Memory Port) Base Address
    pub const RAM_BASE: usize = 0x8000_0000;
    
    /// FLASH size in bytes (512MB)
    pub const FLASH_SIZE: usize = 512 * 1024 * 1024;
    
    /// RAM size in bytes (512MB)
    pub const RAM_SIZE: usize = 512 * 1024 * 1024;
    
    /// ITIM size in bytes (16KB)
    pub const ITIM_SIZE: usize = 16 * 1024;
    
    /// DLS size in bytes (64KB)
    pub const DLS_SIZE: usize = 64 * 1024;
}

/// Hart (Hardware Thread) definitions
pub mod hart {
    /// Number of harts in the system
    pub const HART_COUNT: usize = 4;
    
    /// Stack size per hart in bytes (128KB)
    pub const STACK_SIZE: usize = 128 * 1024;
    
    /// Hart IDs
    pub const HART0: usize = 0;
    pub const HART1: usize = 1;
    pub const HART2: usize = 2;
    pub const HART3: usize = 3;
}

/// UART0 Driver for SiFive-U
/// Simple UART implementation for console output
pub struct Uart0;

/// UART0 Register offsets
pub mod uart0 {
    use super::UART0_BASE;
    
    /// Transmit Data Register
    pub const TXDATA: usize = UART0_BASE + 0x0;
    
    /// Receive Data Register
    pub const RXDATA: usize = UART0_BASE + 0x4;
    
    /// Transmit Control Register
    pub const TXCTRL: usize = UART0_BASE + 0x8;
    
    /// Receive Control Register
    pub const RXCTRL: usize = UART0_BASE + 0xC;
    
    /// Baud Rate Divisor Register
    pub const DIV: usize = UART0_BASE + 0x18;
}

impl Uart0 {
    /// Initialize UART0 - simplified for QEMU
    pub fn init() {
        unsafe {
            // Set divisor for 115200 baud (115200 = 50MHz / (16 * 139))
            let div_ptr = uart0::DIV as *mut u32;
            div_ptr.write_volatile(139);
            
            // Enable transmitter
            let txctrl_ptr = uart0::TXCTRL as *mut u32;
            txctrl_ptr.write_volatile(1); // TXEN bit
            
            // Memory barrier to ensure initialization is visible
            core::arch::asm!("fence iorw, iorw");
        }
    }
    
    /// Send a single byte to UART0 - wait for TX ready
    pub fn send_byte(byte: u8) {
        unsafe {
            // Wait until TX FIFO is not full
            while read_volatile(uart0::TXDATA as *const u32) & 0x80000000 != 0 {
                // TX FIFO full, wait
            }
            
            // Write the byte (lower 8 bits, upper 24 bits should be 0)
            let txdata_ptr_mut = uart0::TXDATA as *mut u32;
            txdata_ptr_mut.write_volatile(byte as u32);
        }
    }
    
    /// Send a string to UART0
    pub fn send_string(s: &str) {
        for byte in s.bytes() {
            Self::send_byte(byte);
        }
    }
    
    /// Check if transmission is complete
    pub fn tx_done() -> bool {
        unsafe {
            let txdata_ptr = uart0::TXDATA as *const u32;
            txdata_ptr.read_volatile() & 0x80000000 == 0
        }
    }
    
    /// Flush any pending transmission
    pub fn flush() {
        while !Self::tx_done() {
            // Wait for transmission to complete
        }
    }
}
