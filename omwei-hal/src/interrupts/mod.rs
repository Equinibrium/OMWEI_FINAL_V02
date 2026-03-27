//! Interrupt handling for OMWEI Equinibrium SoC
//! 
//! This module provides interrupt handling abstractions including
//! inter-processor interrupts (IPI) via CLINT and advanced platform-level
//! interrupt control via APLIC.

pub mod aplic;
pub mod ipi;

pub use aplic::{Aplic, SourceConfig, TargetConfig};
pub use ipi::{send_ipi, clear_msi, enable_msi, disable_msi, is_msi_pending, wait_for_interrupt};
