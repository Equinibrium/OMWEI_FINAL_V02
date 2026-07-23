# OMWEI Atom

Reference implementation of the OMWEI execution integrity format.

## Overview

OMWEI defines a hardware-oriented execution integrity signal for verifiable system behavior. This repository provides the software-level reference implementation of the OMWEI Atom format, which represents structured execution data using the 32BSA (32-bit execution data format) concept.

## What this repository is

- Software-level reference implementation of the OMWEI Atom format
- Represents structured execution data (32BSA concept) 
- Demonstrates how execution context, constraints, and intent are encoded
- Experimental reference implementation aligned with OMWEI Specification v0.1

## Core Concept

A system using OMWEI concepts ensures:

- execution is validated before execution
- constraints are explicitly defined
- execution state is traceable

## 32BSA Format

32BSA represents structured execution data containing:

- intent
- constraints  
- execution context

The format uses a 32-bit Global ID where Bit 31 indicates the execution integrity level:

- **Managed Space (Bit 31 = 0):** Range 0x00000000 - 0x7FFFFFFF
  - Requires post-quantum signature verification
  - For production systems with global execution integrity

- **Community Space (Bit 31 = 1):** Range 0x80000000 - 0xFFFFFFFF  
  - No verification required
  - For development and experimental use

## Performance Characteristics

- **Trust level determination:** Single bit-mask operation (~8.86 nanoseconds)
- **Throughput:** 113 million operations/second
- **Memory:** Zero allocation, stack-only execution
- **Hardware optimization:** Cache-line aligned 32-byte structure

## Usage

### Basic Implementation

```rust
use omwei_atom::trust_hierarchy::{get_trust_level, validate_atom, Atom, TrustLevel};

fn process_execution_data(global_id: u32, payload: [u8; 28]) {
    let atom = Atom::new(global_id, payload);
    
    // Zero-latency execution integrity determination
    let trust_level = get_trust_level(global_id);
    
    match trust_level {
        TrustLevel::Managed => {
            // Production-grade execution with verification
            match validate_atom(&atom) {
                ValidationResult::Trusted => {
                    // Execute with global integrity guarantees
                }
                ValidationResult::InvalidSignature => {
                    // Handle execution integrity failure
                }
                _ => unreachable!(),
            }
        }
        TrustLevel::Community => {
            // Development/experimental execution
            // Local logging only, no global integrity guarantees
        }
    }
}
```

## Status

Experimental reference implementation aligned with OMWEI Specification v0.1

## Documentation

- **API Reference:** [docs.rs/omwei-atom](https://docs.rs/omwei-atom)
- **Trust Hierarchy:** [TRUST_HIERARCHY.md](TRUST_HIERARCHY.md)

## License

Licensed under the **MIT OR Apache-2.0** license.

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
omwei-atom = "0.1.2"
```

## Configuration

### Feature Flags
- `std`: Standard library support (default)
- `serde`: Serialization/deserialization support
- `pqc`: Post-quantum cryptography primitives

### Embedded Targets
```toml
[dependencies.omwei-atom]
version = "0.1.2"
default-features = false
features = ["pqc"]  # For bare-metal environments
```
