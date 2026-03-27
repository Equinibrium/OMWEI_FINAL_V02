//! Cryptographic hardware drivers for OMWEI Equinibrium SoC
//! 
//! This module provides drivers for the Hardware Crypto Accelerator (HCA)
//! peripherals, including the True Random Number Generator (TRNG) and
//! AES-256 encryption engine.

pub mod trng;

pub use trng::Trng;
