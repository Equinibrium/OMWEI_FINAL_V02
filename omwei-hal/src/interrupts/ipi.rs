//! Inter-Processor Interrupt (IPI) handling for OMWEI Equinibrium SoC
//! 
//! This module provides safe abstractions for sending machine software
//! interrupts between harts using the CLINT MSIP registers.

use crate::registers::clint;

/// Send an Inter-Processor Interrupt to a target hart
/// 
/// # Arguments
/// * `target_hart` - The hart ID to send the IPI to (0-3)
/// 
/// # Safety
/// This function is safe as it only writes to the MSIP register which is
/// designed for this purpose. The write is atomic and uses proper memory
/// barriers to ensure consistency across the 128-bit memory port.
pub fn send_ipi(target_hart: usize) {
    if target_hart >= 4 {
        panic!("Invalid hart ID: {}", target_hart);
    }
    
    let msip_addr = clint::msip(target_hart);
    
    // Write 1 to the MSIP register to trigger the interrupt
    // Use volatile write to ensure the write is not optimized away
    unsafe {
        // Memory barrier before IPI to ensure all previous writes are visible
        core::arch::asm!("fence iorw, iorw");
        
        // Write to MSIP register
        let msip_ptr = msip_addr as *mut u32;
        msip_ptr.write_volatile(1);
        
        // Memory barrier after IPI to ensure the interrupt is visible
        core::arch::asm!("fence iorw, iorw");
    }
}

/// Clear the Machine Software Interrupt for the current hart
/// 
/// This should be called in the interrupt handler to clear the pending interrupt.
pub fn clear_msi() {
    let hart_id = riscv::register::mhartid::read();
    let msip_addr = clint::msip(hart_id);
    
    unsafe {
        // Memory barrier before clearing
        core::arch::asm!("fence iorw, iorw");
        
        // Write 0 to clear the interrupt
        let msip_ptr = msip_addr as *mut u32;
        msip_ptr.write_volatile(0);
        
        // Memory barrier after clearing
        core::arch::asm!("fence iorw, iorw");
    }
}

/// Enable Machine Software Interrupts in the MIE register
pub fn enable_msi() {
    unsafe {
        riscv::register::mie::set_msoft();
        // Memory barrier to ensure the interrupt enable is visible
        core::arch::asm!("fence iorw, iorw");
    }
}

/// Disable Machine Software Interrupts in the MIE register
pub fn disable_msi() {
    unsafe {
        riscv::register::mie::clear_msoft();
        // Memory barrier to ensure the interrupt disable is visible
        core::arch::asm!("fence iorw, iorw");
    }
}

/// Check if Machine Software Interrupt is pending for the current hart
pub fn is_msi_pending() -> bool {
    let hart_id = riscv::register::mhartid::read();
    let msip_addr = clint::msip(hart_id);
    
    unsafe {
        let msip_ptr = msip_addr as *const u32;
        msip_ptr.read_volatile() != 0
    }
}

/// Wait for an interrupt using the WFI instruction
/// 
/// This puts the hart in a low-power state until an interrupt arrives.
/// The function returns after the interrupt is handled.
pub fn wait_for_interrupt() {
    unsafe {
        // Memory barrier before WFI to ensure all operations are complete
        core::arch::asm!("fence iorw, iorw");
        
        // Wait for interrupt
        core::arch::asm!("wfi");
        
        // Memory barrier after WFI to ensure we see updated state
        core::arch::asm!("fence iorw, iorw");
    }
}
