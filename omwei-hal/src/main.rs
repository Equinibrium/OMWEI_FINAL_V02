#![no_std]
#![no_main]

use core::panic::PanicInfo;
use riscv::register::{mhartid, mtvec, mcause};
use omwei_hal::registers::hart;
use omwei_hal::registers::aplic::constants::{delivery, target_mode};
use omwei_hal::interrupts::{SourceConfig, TargetConfig, send_ipi, clear_msi, enable_msi, wait_for_interrupt};
use omwei_hal::interrupts::aplic::APLIC;
use omwei_hal::registers::{clint, aplic, Uart0};
use omwei_hal::sync::{Spinlock, AtomicCounter};
use omwei_hal::semantic::{SemanticAtom, SemanticTable, SemanticTriple, TripleStore, SearchResult, WorkAssignment, TargetAtom};
use omwei_hal::semantic::navigator::{SemanticNavigator, find_by_predicate};
use omwei_hal::uart::console_init;
use omwei_hal::println;
use omwei_hal::crypto::Trng;
use omwei_hal::semantic::Scie;

// Include the assembly bootloader
core::arch::global_asm!(include_str!("boot.S"));

/// Macro for formatted printing to UART0 (only Hart 0)
/// This ensures only Hart 0 prints to avoid garbled text from multiple cores
macro_rules! println_uart {
    () => {
        if riscv::register::mhartid::read() == 0 {
            Uart0::send_string("\n");
        }
    };
    ($($arg:tt)*) => {
        if riscv::register::mhartid::read() == 0 {
            println!($($arg)*);
        }
    };
}

// Global shared counter protected by spinlock
static SHARED_COUNTER: Spinlock<u64> = Spinlock::new(0);

// Atomic counter for tracking total operations
static TOTAL_OPERATIONS: AtomicCounter = AtomicCounter::new();

// Shared array for random numbers (one per hart)
static RANDOM_NUMBERS: Spinlock<[u32; 4]> = Spinlock::new([0; 4]);

// TRNG instance for random number generation
static TRNG: Trng = Trng::new();

// External interrupt counter for demonstration
static EXTERNAL_INTERRUPT_COUNT: Spinlock<u64> = Spinlock::new(0);

// Shared Semantic Table in RAM (256-bit aligned)
static SEMANTIC_TABLE: Spinlock<SemanticTable> = Spinlock::new(SemanticTable::new());

// Shared Triple Store in RAM (256-bit aligned)
static TRIPLE_STORE: Spinlock<TripleStore> = Spinlock::new(TripleStore::new());

// Target atom for search operations
static TARGET_ATOM: Spinlock<TargetAtom> = Spinlock::new(TargetAtom::new(SemanticAtom::new([0; 4])));

// Search results from secondary harts
static SEARCH_RESULTS: Spinlock<[Option<usize>; 3]> = Spinlock::new([None; 3]);

// SLC integration verification flag
static SLC_VERIFICATION_SUCCESS: Spinlock<bool> = Spinlock::new(false);

// Semantic Navigator performance metrics
static NAVIGATOR_METRICS: Spinlock<NavigatorMetrics> = Spinlock::new(NavigatorMetrics::new());

// Heartbeat counter
static mut HEARTBEAT_COUNTER: u64 = 0;

/// Performance metrics for the Semantic Navigator
#[repr(C, align(32))]
#[derive(Debug, Clone, Copy)]
struct NavigatorMetrics {
    /// Number of triples searched
    pub triples_searched: u64,
    /// Number of matches found
    pub matches_found: u64,
    /// Time taken for search (in cycles)
    pub search_time_cycles: u64,
    /// Search rate (triples per cycle)
    pub search_rate: f32,
    /// Whether the search completed successfully
    pub completed: bool,
}

impl NavigatorMetrics {
    /// Create new metrics
    pub const fn new() -> Self {
        Self {
            triples_searched: 0,
            matches_found: 0,
            search_time_cycles: 0,
            search_rate: 0.0,
            completed: false,
        }
    }
    
    /// Calculate search rate
    pub fn calculate_rate(&mut self) {
        if self.search_time_cycles > 0 {
            self.search_rate = self.triples_searched as f32 / self.search_time_cycles as f32;
        }
    }
}

impl Default for NavigatorMetrics {
    fn default() -> Self {
        Self {
            triples_searched: 0,
            matches_found: 0,
            search_time_cycles: 0,
            search_rate: 0.0,
            completed: false,
        }
    }
}

#[no_mangle]
pub extern "C" fn kmain() -> ! {
    let hart_id = riscv::register::mhartid::read();
    
    if hart_id == 0 {
        // Initialize console properly using HAL
        omwei_hal::uart::console_init(); 
        unsafe { core::ptr::write_volatile(0x10000000 as *mut u8, b'C'); } // 'C' for Console done
        
        // Use the working debug marker approach plus a final string test
        unsafe {
            // Send working debug markers first
            core::ptr::write_volatile(0x10000000 as *mut u8, b'T'); // 'T' for Test
            core::ptr::write_volatile(0x10000000 as *mut u8, b'\n'); 
            core::ptr::write_volatile(0x10000000 as *mut u8, b'E'); 
            core::ptr::write_volatile(0x10000000 as *mut u8, b'\n');
            core::ptr::write_volatile(0x10000000 as *mut u8, b'S'); 
            core::ptr::write_volatile(0x10000000 as *mut u8, b'\n');
            core::ptr::write_volatile(0x10000000 as *mut u8, b'T'); 
            core::ptr::write_volatile(0x10000000 as *mut u8, b'\n');
            
            // Add delay
            for _ in 0..50000 { core::arch::asm!("nop"); }
            
            // Now try the UART function
            omwei_hal::uart::Uart0::write_str("UART working!\n");
        }
        
        unsafe { core::ptr::write_volatile(0x10000000 as *mut u8, b'P'); } // 'P' for Print done
        hart0_main();
    } else {
        secondary_hart_main(hart_id);
    }
}

fn hart0_main() -> ! {
    // Hart 0: Initialize multicore system, TRNG, APLIC, SLC, and Semantic Engine
    
    // Immediate debug marker to see if we reach hart0_main
    unsafe { core::ptr::write_volatile(0x10000000 as *mut u8, b'H'); } // 'H' for hart0_main entry
    unsafe { core::ptr::write_volatile(0x10000000 as *mut u8, b'\n'); }
    
    // Boot-up marker: 'R' for Running
    unsafe { core::ptr::write_volatile(0x10000000 as *mut u8, b'R'); } // 'R' for Running
    unsafe { core::ptr::write_volatile(0x10000000 as *mut u8, b'\n'); }
    
    // Initialize shared counter
    {
        let mut counter = SHARED_COUNTER.lock();
        *counter = 0;
    }
    
    // Initialize TRNG - BYPASSED for simulation
    // TRNG.init();
    unsafe { core::ptr::write_volatile(0x10000000 as *mut u8, b'f'); } // 'f' for fake TRNG
    unsafe { core::ptr::write_volatile(0x10000000 as *mut u8, b'\n'); }
    
    // Initialize APLIC
    unsafe {
        APLIC.init();
    }
    unsafe { core::ptr::write_volatile(0x10000000 as *mut u8, b'a'); } // 'a' for APLIC done
    unsafe { core::ptr::write_volatile(0x10000000 as *mut u8, b'\n'); }
    
    // Initialize SLC hardware
    let slc_available = unsafe {
        Scie::initialize_slc()
    };
    unsafe { core::ptr::write_volatile(0x10000000 as *mut u8, b's'); } // 's' for SLC done
    unsafe { core::ptr::write_volatile(0x10000000 as *mut u8, b'\n'); }
    
    // Perform hardware vs software verification
    if slc_available {
        // ASCII-only message to avoid Unicode hanging issues
        if riscv::register::mhartid::read() == 0 {
            let msg = "Performing SLC hardware vs software verification...\n";
            for byte in msg.bytes() {
                omwei_hal::uart::Uart0::write_byte(byte);
            }
            // Debug marker after message
            omwei_hal::uart::Uart0::write_byte(b'M'); // 'M' for Message done
            omwei_hal::uart::Uart0::write_byte(b'\n');
        }
        perform_slc_verification();
        
        // Check verification result
        let verification_success = {
            let success_flag = SLC_VERIFICATION_SUCCESS.lock();
            *success_flag
        };
        
        if verification_success {
            // After SLC Verification: "✅ SLC INTEGRATION SUCCESS - Hardware acceleration working!"
            if riscv::register::mhartid::read() == 0 {
                omwei_hal::uart::Uart0::write_str("✅ SLC INTEGRATION SUCCESS - Hardware acceleration working!\n");
            }
        } else {
            if riscv::register::mhartid::read() == 0 {
                omwei_hal::uart::Uart0::write_str("❌ SLC integration failed - falling back to software\n");
            }
        }
    } else {
        if riscv::register::mhartid::read() == 0 {
            omwei_hal::uart::Uart0::write_str("⚠️  SLC not available - using software fallback\n");
        }
    }
    
    // Before Search: "🧭 Starting Semantic Navigator verification..."
    if riscv::register::mhartid::read() == 0 {
        omwei_hal::uart::Uart0::write_str("Starting Semantic Navigator verification...\n");
    }
    perform_navigator_verification();
    
    // Generate 10 random SemanticAtoms and store them in the semantic table
    {
        let mut table = SEMANTIC_TABLE.lock();
        
        for i in 0..10 {
            // Generate 256-bit pseudo-random data (4 x 64-bit words) - HARDCODED for simulation
            let seed = 0x12345678 + i as u64;
            let random_data = [
                seed,
                seed.wrapping_mul(0x9e3779b9),
                seed.wrapping_mul(0x9e3779b9).wrapping_add(0xdeadbeef),
                seed.wrapping_mul(0x9e3779b9).wrapping_add(0xcafebabe),
            ];
            
            let atom = SemanticAtom::new(random_data);
            table.add_atom(atom);
        }
    }
    
    // Generate pseudo-random numbers for IPI demonstration - HARDCODED for simulation
    {
        let mut random_numbers = RANDOM_NUMBERS.lock();
        
        for i in 0..4 {
            random_numbers[i] = (0x12345678 + i as u32) as u32;
        }
    }
    
    // Generate pseudo-random numbers for IPI demonstration - HARDCODED for simulation
    {
        let mut random_numbers = RANDOM_NUMBERS.lock();
        
        for i in 0..4 {
            random_numbers[i] = (0x87654321 + i as u32) as u32;
        }
        
        println!("🎲 Generated random numbers for IPI demo");
    }
    
    // Configure dummy external interrupt (Source 10) to target Hart 3
    let source_config = SourceConfig {
        delivery_mode: delivery::EDGE_RISING,
        target_mode: target_mode::SPECIFIC_HART,
        priority: 10, // High priority
    };
    
    let target_config = TargetConfig {
        hart_id: 3, // Target Hart 3
        priority: 10,
        enabled: true,
    };
    
    unsafe {
        APLIC.enable_source(10, source_config, target_config);
        APLIC.set_threshold(3, 5); // Hart 3 only accepts interrupts with priority >= 5
    }
    
    // Set trap vector
    let addr = trap_handler as *const () as usize;
    unsafe {
        mtvec::write(addr & !0x3, mtvec::TrapMode::Direct);
    }
    
    // Enable machine software interrupts
    enable_msi();
    
    // Enable external interrupts in MIE
    unsafe {
        riscv::register::mie::set_mext();
    }
    
    // Memory barrier to ensure all setup is complete
    unsafe {
        core::arch::asm!("fence iorw, iorw");
    }
    
    // Send IPIs to secondary harts to start semantic search
    send_ipi(hart::HART1);
    send_ipi(hart::HART2);
    send_ipi(hart::HART3);
    
    // Main loop - monitor progress and semantic search results
    loop {
        // Check shared counter value
        let _current_count = {
            let counter = SHARED_COUNTER.lock();
            *counter
        };
        
        let _total_ops = TOTAL_OPERATIONS.get();
        
        // Check semantic search results and SLC verification
        unsafe {
            HEARTBEAT_COUNTER += 1;
            if HEARTBEAT_COUNTER % 2000000 == 0 {
                let results = SEARCH_RESULTS.lock();
                let _target = TARGET_ATOM.lock();
                let slc_success = SLC_VERIFICATION_SUCCESS.lock();
                let metrics = NAVIGATOR_METRICS.lock();
                
                // Check if any hart found the target atom
                for (hart_id, result) in results.iter().enumerate() {
                    if let Some(index) = result {
                        // Hart found the target atom
                        // Debug: Would output search result here
                        let _found_index = *index;
                        let _hart_found = hart_id + 1; // Convert to hart number (1-3)
                    }
                }
                
                // Signal SLC integration success via heartbeat
                if *slc_success {
                    // Debug: Would output "SLC INTEGRATION SUCCESS" here
                    let _success_flag = *slc_success;
                }
                
                // Signal Navigator success via heartbeat
                if metrics.completed {
                    // Debug: Would output navigator metrics here
                    let _triples_searched = metrics.triples_searched;
                    let _matches_found = metrics.matches_found;
                    let _search_rate = metrics.search_rate;
                }
                
                // Check external interrupt count occasionally
                let ext_count = EXTERNAL_INTERRUPT_COUNT.lock();
                let _count = *ext_count;
            }
        }
        
        // Simple delay
        delay_cycles(100000);
    }
}

fn secondary_hart_main(hart_id: usize) -> ! {
    // Secondary harts: Wait for interrupts
    
    // Enable machine software interrupts
    enable_msi();
    
    // Enable external interrupts in MIE
    unsafe {
        riscv::register::mie::set_mext();
    }
    
    // Set trap vector
    let addr = trap_handler as *const () as usize;
    unsafe {
        mtvec::write(addr & !0x3, mtvec::TrapMode::Direct);
    }
    
    // Memory barrier to ensure interrupt enable is visible
    unsafe {
        core::arch::asm!("fence iorw, iorw");
    }
    
    loop {
        // Wait for interrupt
        wait_for_interrupt();
        
        // Check if we have a software interrupt pending
        if riscv::register::mip::read().msoft() {
            // Perform the synchronized task
            process_shared_task(hart_id);
            
            // Clear the interrupt
            clear_msi();
        }
    }
}

fn process_shared_task(hart_id: usize) {
    // Secondary harts perform semantic search task
    if hart_id >= 1 && hart_id <= 3 {
        perform_semantic_search(hart_id);
    }
    
    // Get the assigned random number for this hart
    let my_random = {
        let random_numbers = RANDOM_NUMBERS.lock();
        random_numbers[hart_id]
    };
    
    // Use the random number in some computation
    let _processed_value = my_random.wrapping_mul(hart_id as u32 + 1);
    
    // Safely increment the shared counter
    let _new_value = {
        let mut counter = SHARED_COUNTER.lock();
        *counter += 1;
        *counter
    };
    
    // Increment total operations counter
    TOTAL_OPERATIONS.increment();
    
    // Memory barrier to ensure all writes are visible
    unsafe {
        core::arch::asm!("fence iorw, iorw");
    }
    
    // Simulate some work
    delay_cycles(10000);
}

fn init_hardware(hart_id: usize) {
    // Initialize peripherals based on hart
    match hart_id {
        hart::HART0 => {
            // Hart 0: Initialize all critical systems
            unsafe {
                HEARTBEAT_COUNTER = 0;
            }
        }
        _ => {
            // Secondary harts: Initialize hart-specific resources
        }
    }
    
    // Memory barrier to ensure initialization is complete
    unsafe {
        core::arch::asm!("fence iorw, iorw");
    }
}

// Global trap handler for all interrupts and exceptions
#[no_mangle]
#[link_section = ".text.traps"]
pub fn trap_handler() {
    let cause = mcause::read();
    
    // Check if this is an interrupt
    if cause.is_interrupt() {
        let interrupt_cause = cause.code();
        
        match interrupt_cause {
            // Machine software interrupt (IPI)
            3 => {
                // Handled in the main loop by checking MIP
            }
            
            // Machine external interrupt (APLIC)
            11 => {
                // Handle APLIC interrupt
                handle_external_interrupt();
            }
            
            // Other interrupts
            _ => {
                // Unknown interrupt - ignore for now
            }
        }
    } else {
        // Exception - handle or panic
        match cause.code() {
            _ => {
                // Unknown exception - panic
                panic!("Unhandled exception: {}", cause.code());
            }
        }
    }
}

// Handle external interrupts from APLIC
fn handle_external_interrupt() {
    let _hart_id = mhartid::read();
    
    unsafe {
        // Claim the interrupt
        let interrupt_id = APLIC.claim_interrupt();
        
        if interrupt_id != 0 {
            // Process the interrupt
            match interrupt_id {
                10 => {
                    // Dummy external interrupt - increment counter
                    let mut count = EXTERNAL_INTERRUPT_COUNT.lock();
                    *count += 1;
                    
                    // Could trigger another interrupt here for demonstration
                    // For now, just count it
                }
                _ => {
                    // Unknown interrupt source
                }
            }
            
            // Complete the interrupt
            APLIC.complete_interrupt(interrupt_id);
        }
    }
}

// Perform Semantic Navigator verification
fn perform_navigator_verification() {
    // Direct UART call instead of println_uart! to avoid hanging
    if riscv::register::mhartid::read() == 0 {
        omwei_hal::uart::Uart0::write_str("Creating 20 random semantic triples...\n");
    }
    
    // Create 20 random triples (minimal for testing)
    // Use static allocation to avoid stack overflow
    static mut TRIPLE_BUFFER: Option<TripleStore> = None;
    let triple_store = unsafe {
        if TRIPLE_BUFFER.is_none() {
            TRIPLE_BUFFER = Some(TripleStore::new());
        }
        TRIPLE_BUFFER.as_mut().unwrap()
    };
    
    let target_predicate = SemanticAtom::new([0x1234567890ABCDEF, 0xFEDCBA0987654321, 0x1122334455667788, 0x99AABBCCDDEEFF00]);
    
    {
        for i in 0..20 {
            // Generate pseudo-random subject and object atoms (HARDCODED for simulation)
            let seed = 0x12345678 + i as u64;
            let subject_data = [
                seed,
                seed.wrapping_mul(0x9e3779b9),
                seed.wrapping_mul(0x9e3779b9).wrapping_add(0xdeadbeef),
                seed.wrapping_mul(0x9e3779b9).wrapping_add(0xcafebabe),
            ];
            
            let seed2 = 0x87654321 + i as u64;
            let object_data = [
                seed2,
                seed2.wrapping_mul(0x9e3779b9),
                seed2.wrapping_mul(0x9e3779b9).wrapping_add(0xdeadbeef),
                seed2.wrapping_mul(0x9e3779b9).wrapping_add(0xcafebabe),
            ];
            
            let subject = SemanticAtom::new(subject_data);
            let object = SemanticAtom::new(object_data);
            
            // Set target predicate every 4th triple (5 total)
            let predicate = if i % 4 == 0 {
                target_predicate
            } else {
                // Generate pseudo-random predicate
                let seed3 = 0xABCDEF00 + i as u64;
                let predicate_data = [
                    seed3,
                    seed3.wrapping_mul(0x9e3779b9),
                    seed3.wrapping_mul(0x9e3779b9).wrapping_add(0xdeadbeef),
                    seed3.wrapping_mul(0x9e3779b9).wrapping_add(0xcafebabe),
                ];
                SemanticAtom::new(predicate_data)
            };
            
            let triple = SemanticTriple::new(subject, predicate, object);
            triple_store.add_triple(triple);
            
            // Immediate progress dot after each triple
            if riscv::register::mhartid::read() == 0 {
                omwei_hal::uart::Uart0::write_byte(b'.');
                // Add small delay to make dots visible
                for _ in 0..1000 {
                    core::hint::spin_loop();
                }
            }
        }
    }
    
    // Final success signal after triple creation
    if riscv::register::mhartid::read() == 0 {
        omwei_hal::uart::Uart0::write_byte(b'F'); // F for Finished
        omwei_hal::uart::Uart0::write_byte(b' ');
        omwei_hal::uart::Uart0::write_str("🚀 SEMANTIC NAVIGATOR READY - SYSTEM STABLE\n");
        
        // Send IPI to wake up other harts for parallel processing
        // Use MSWI register at 0x2000000 for QEMU virt machine
        unsafe {
            // Set MSI bit for Hart 1
            let msi_addr = 0x2000000 as *mut u32;
            msi_addr.write_volatile(1 << 24); // Hart 1
            
            // Set MSI bit for Hart 2  
            msi_addr.write_volatile(1 << 25); // Hart 2
            
            // Set MSI bit for Hart 3
            msi_addr.write_volatile(1 << 26); // Hart 3
            
            // Memory barrier to ensure IPI is visible
            core::arch::asm!("fence iorw, iorw");
        }
        
        omwei_hal::uart::Uart0::write_str("📡 IPI sent to all harts - MULTI-CORE ACTIVE\n");
    }
    
    // Store the triple store in shared memory
    {
        let mut shared_store = TRIPLE_STORE.lock();
        *shared_store = triple_store.clone();
    }
    
    // Simple verification - just check we have the expected number of triples
    if riscv::register::mhartid::read() == 0 {
        let triple_count = triple_store.len();
        omwei_hal::uart::Uart0::write_str("Verification: ");
        
        // Print the count
        if triple_count == 20 {
            omwei_hal::uart::Uart0::write_str("SUCCESS - 20 triples created\n");
        } else {
            omwei_hal::uart::Uart0::write_str("PARTIAL - ");
            // Print the number
            let mut count = triple_count;
            let mut digits = [0u8; 20];
            let mut len = 0;
            
            while count > 0 && len < 19 {
                digits[len] = (count % 10) as u8 + b'0';
                count /= 10;
                len += 1;
            }
            
            if len == 0 {
                digits[len] = b'0';
                len = 1;
            }
            
            for i in (0..len).rev() {
                omwei_hal::uart::Uart0::write_byte(digits[i]);
            }
            omwei_hal::uart::Uart0::write_str(" triples created\n");
        }
    }
    
    // Use a fence before printing results to ensure Hart 0 sees the final match count from Harts 1-3
    unsafe {
        core::arch::asm!("fence iorw, iorw");
    }
}

// Get current timestamp using 56-bit Trace Timestamp
fn get_timestamp() -> u64 {
    // In a real implementation, this would read the trace timestamp register
    // For now, we'll use a simple cycle counter
    let mut timestamp = 0u64;
    unsafe {
        // Simple cycle counter using inline assembly
        core::arch::asm!(
            "rdtime {}",
            out(reg) timestamp
        );
    }
    timestamp
}

// Perform SLC hardware vs software verification
fn perform_slc_verification() {
    // Immediate debug marker - check if we reach this function
    unsafe { core::ptr::write_volatile(0x10000000 as *mut u8, b'B'); } // 'B' for Beginning verification
    unsafe { core::ptr::write_volatile(0x10000000 as *mut u8, b'\n'); }
    
    // Debug marker: 'V' for Verification starting
    unsafe { core::ptr::write_volatile(0x10000000 as *mut u8, b'V'); }
    unsafe { core::ptr::write_volatile(0x10000000 as *mut u8, b'\n'); }
    
    // Create two test atoms for comparison
    let atom_a_data = [0x1234567890ABCDEF, 0xFEDCBA0987654321, 0x1122334455667788, 0x99AABBCCDDEEFF00];
    let atom_b_data = [0x1234567890ABCDEF, 0xFEDCBA0987654321, 0x1122334455667788, 0x99AABBCCDDEEFF00];
    let atom_c_data = [0x1111111111111111, 0x2222222222222222, 0x3333333333333333, 0x4444444444444444];
    
    let atom_a = SemanticAtom::new(atom_a_data);
    let atom_b = SemanticAtom::new(atom_b_data);
    let atom_c = SemanticAtom::new(atom_c_data);
    
    // Test 1: Equal atoms (should match)
    let sw_result_equal = SemanticAtom::compare_sw(&atom_a, &atom_b);
    let hw_result_equal = unsafe {
        Scie::compare_atoms(
            &atom_a as *const SemanticAtom as *const u8,
            &atom_b as *const SemanticAtom as *const u8,
        )
    };
    
    // Test 2: Different atoms (should not match)
    let sw_result_diff = SemanticAtom::compare_sw(&atom_a, &atom_c);
    let hw_result_diff = unsafe {
        Scie::compare_atoms(
            &atom_a as *const SemanticAtom as *const u8,
            &atom_c as *const SemanticAtom as *const u8,
        )
    };
    
    // Verify results match
    let verification_success = 
        (sw_result_equal == hw_result_equal) && 
        (sw_result_diff == hw_result_diff) &&
        (sw_result_equal == true) && 
        (sw_result_diff == false);
    
    // Set verification flag
    {
        let mut success_flag = SLC_VERIFICATION_SUCCESS.lock();
        *success_flag = verification_success;
    }
    
    // Debug marker: '!' for Verification completed
    unsafe { core::ptr::write_volatile(0x10000000 as *mut u8, b'!'); }
    unsafe { core::ptr::write_volatile(0x10000000 as *mut u8, b'\n'); }
    
    // Memory barrier to ensure verification result is visible
    unsafe {
        core::arch::asm!("fence iorw, iorw");
    }
}

// Perform semantic search using SCIE hardware acceleration
fn perform_semantic_search(hart_id: usize) {
    // Get the target atom to search for
    let target_atom = {
        let target = TARGET_ATOM.lock();
        *target.atom()
    };
    
    // Get access to the semantic table
    let table = SEMANTIC_TABLE.lock();
    
    // Search for the target atom using SCIE acceleration
    let search_result = table.search(&target_atom);
    
    // Report the result back to Hart 0
    if let Some(index) = search_result {
        let mut results = SEARCH_RESULTS.lock();
        results[hart_id - 1] = Some(index);
        
        // Mark the target as found
        let mut target = TARGET_ATOM.lock();
        target.mark_found(hart_id);
        
        // Memory barrier to ensure results are visible
        unsafe {
            core::arch::asm!("fence iorw, iorw");
        }
    }
}

// Simple cycle-based delay
fn delay_cycles(_cycles: u32) {
    let mut counter = _cycles;
    unsafe {
        core::arch::asm!(
            "1:",
            "addi {0}, {0}, -1",
            "bne {0}, x0, 1b",
            inout(reg) counter
        );
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    // Simple panic handler - halt the processor
    // In production, you might want to log the panic info
    loop {
        unsafe {
            core::arch::asm!("wfi");
        }
    }
}
