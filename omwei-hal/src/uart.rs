//! UART0 Driver for NS16550A UART (QEMU virt machine)
//! 
//! UART0 is mapped at 0x10000000 on QEMU virt machine (NS16550A)
//! Used for console output and debugging

use core::fmt;
use core::ptr::{read_volatile, write_volatile};

/// UART0 base address on QEMU virt machine (NS16550A)
const UART0_BASE: usize = 0x10000000;

/// NS16550A UART register offsets
const UART_THR: usize = 0x0;  // Transmitter Holding Register
const UART_IER: usize = 0x1;  // Interrupt Enable Register  
const UART_FCR: usize = 0x2;  // FIFO Control Register
const UART_LCR: usize = 0x3;  // Line Control Register
const UART_LSR: usize = 0x5;  // Line Status Register

/// NS16550A Line Control Register - 8N1 configuration
const LCR_8N1: u8 = 0x03; // 8 data bits, no parity, 1 stop bit

/// NS16550A FIFO Control Register - Enable FIFO
const FCR_ENABLE: u8 = 0x07; // Enable FIFO, clear TX/RX FIFO

/// NS16550A Line Status Register - Transmitter Empty
const LSR_TX_EMPTY: u8 = 0x20; // Bit 5: Transmitter Holding Register Empty

/// NS16550A Divisor Latch Access bit (in LCR)
const LCR_DLAB: u8 = 0x80; // Bit 7: Divisor Latch Access

/// Baud rate divisor for 115200 (assuming 1.8432 MHz clock)
/// divisor = clock_freq / (16 * baud_rate)
/// divisor = 1_843_200 / (16 * 115200) = 1
const BAUD_DIVISOR: u16 = 1;

/// UART structure for console operations
pub struct Uart0;

impl Uart0 {
    /// Initialize UART0 for console output
    /// 
    /// Sets up NS16550A UART with 8N1 configuration, FIFO enabled, and 115200 baud
    pub fn init() {
        unsafe {
            // Step 1: Set DLAB bit to access baud rate divisor
            let lcr_ptr = (UART0_BASE + UART_LCR) as *mut u8;
            lcr_ptr.write_volatile(LCR_8N1 | LCR_DLAB);
            
            // Step 2: Set baud rate divisor (DLL and DLM registers)
            let dll_ptr = (UART0_BASE + UART_THR) as *mut u8; // DLL = THR when DLAB=1
            let dlm_ptr = (UART0_BASE + UART_IER) as *mut u8; // DLM = IER when DLAB=1
            dll_ptr.write_volatile((BAUD_DIVISOR & 0xFF) as u8);
            dlm_ptr.write_volatile((BAUD_DIVISOR >> 8) as u8);
            
            // Step 3: Clear DLAB bit and set 8N1 configuration
            lcr_ptr.write_volatile(LCR_8N1);
            
            // Step 4: Enable FIFO with FIFO Control Register
            let fcr_ptr = (UART0_BASE + UART_FCR) as *mut u8;
            fcr_ptr.write_volatile(FCR_ENABLE);
            
            // Step 5: Disable interrupts (set IER to 0)
            let ier_ptr = (UART0_BASE + UART_IER) as *mut u8;
            ier_ptr.write_volatile(0);
            
            // Memory barrier to ensure initialization is visible
            core::arch::asm!("fence iorw, iorw");
        }
    }
    
    /// Write a single byte to UART0
    /// 
    /// # Arguments
    /// * `byte` - Byte to transmit
    pub fn write_byte(byte: u8) {
        unsafe {
            // Wait until transmitter is ready (Transmitter Holding Register Empty)
            // Add timeout to prevent hanging on blocked registers
            let mut timeout = 100_000; // 100,000 cycles timeout
            while read_volatile((UART0_BASE + UART_LSR) as *const u8) & LSR_TX_EMPTY == 0 {
                if timeout == 0 {
                    // Timeout reached, skip this byte to avoid hanging
                    return;
                }
                timeout -= 1;
                core::hint::spin_loop();
            }
            
            // Write the byte to Transmitter Holding Register
            let thr_ptr = (UART0_BASE + UART_THR) as *mut u8;
            thr_ptr.write_volatile(byte);
        }
    }
    
    /// Write a string to UART0
    /// 
    /// # Arguments
    /// * `s` - String to transmit
    pub fn write_str(s: &str) {
        for byte in s.bytes() {
            Self::write_byte(byte);
        }
    }
    
    /// Flush any pending transmission
    pub fn flush() {
        unsafe {
            // Wait until transmitter is idle (both FIFO empty and transmitter ready)
            while read_volatile((UART0_BASE + UART_LSR) as *const u8) & LSR_TX_EMPTY == 0 {
                // Transmitter busy, wait
                core::hint::spin_loop();
            }
        }
    }
    
    /// Check if transmission is complete
    pub fn tx_done() -> bool {
        unsafe {
            read_volatile((UART0_BASE + UART_LSR) as *const u8) & LSR_TX_EMPTY != 0
        }
    }
}

/// Implement Write trait for console formatting
impl fmt::Write for Uart0 {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        Self::write_str(s);
        Ok(())
    }
}

/// Global console writer
pub static mut CONSOLE: Uart0 = Uart0;

/// Initialize console for debug output
pub fn console_init() {
    Uart0::init();
}

/// Print to console using format! macro
/// 
/// # Arguments
/// * `args` - Formatted arguments to print
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ({
        use core::fmt::Write;
        unsafe {
            let _ = write!(&mut $crate::uart::CONSOLE, $($arg)*);
        }
    });
}

/// Print to console with newline
/// 
/// # Arguments
/// * `args` - Formatted arguments to print
#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ({
        $crate::print!($($arg)*);
        $crate::print!("\n");
    });
}
