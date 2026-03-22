# OMWEI 32B Semantic Atom - Core Implementation for Agentic AI and Industrial IoT Integrity

[![Crates.io](https://img.shields.io/crates/v/omwei-atom.svg)](https://crates.io/crates/omwei-atom)
[![Documentation](https://docs.rs/omwei-atom/badge.svg)](https://docs.rs/omwei-atom)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)](https://www.rust-lang.org)

The **OMWEI 32B Semantic Atom** is the core implementation of the 32BSA standard for semantic interoperability in industrial AI systems. This reference implementation provides the foundational building blocks for creating intelligent, autonomous industrial ecosystems where AI agents can seamlessly understand and process telemetry data from any source.

## 🎯 The 32BSA Vision

32BSA represents a paradigm shift in industrial data exchange:

- **🧠 Semantic Universality**: One format to describe all industrial telemetry
- **🤖 AI-Native Design**: Built from the ground up for agentic AI systems
- **⚡ Quantum-Ready**: Post-quantum cryptographic anchors for future-proof security
- **🔧 Hardware-Optimized**: 32-byte fixed layout for embedded and edge deployment
- **🌐 Universal Interoperability**: Cross-vendor, cross-protocol data exchange

## 🏗️ Architecture

### The Atom Philosophy

Each 32-byte atom is a self-contained semantic packet that encapsulates:
- **Entity Identification** (4 bytes) - Unique source identifier
- **Telemetry Metadata** (4 bytes) - Type classification and processing flags  
- **Semantic Validation** (4 bytes) - AI-interpretable quality and meaning
- **Measurement Value** (4 bytes) - Fixed-point precision data
- **Temporal Context** (8 bytes) - Microsecond-precision timestamp
- **Trust & Classification** (4 bytes) - Quality scoring and categorization
- **Cryptographic Anchor** (4 bytes) - Post-quantum signature reference

### Memory Layout

```text
Offset | Size | Field           | Description
-------|------|-----------------|-------------
0      | 4    | entity_id       | Unique entity identifier
4      | 4    | telemetry_info  | Type (16b) + Flags (16b)
8      | 4    | predicate_id    | Semantic validation result
12     | 4    | value_mm        | Measurement value (hundredths)
16     | 8    | timestamp_us    | Unix timestamp in microseconds
24     | 4    | tag_trust       | Tag ID (24b) + Trust (8b)
28     | 4    | pqc_anchor      | PQC signature anchor
```

## � Quick Start

### Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
omwei-atom = "0.1"
```

For embedded/no-std environments:

```toml
[dependencies]
omwei-atom = { version = "0.1", default-features = false }
```

### Basic Usage

```rust
use bsa_types::{SemanticAtom, TELEMETRY_TEMPERATURE_C, TRUST_VERIFIED};

// Create a temperature measurement
let atom = SemanticAtom::new(
    0x12345678,                    // entity_id
    TELEMETRY_TEMPERATURE_C,       // telemetry_type
    0x00000001,                    // predicate_id (normal)
    2345,                          // value_mm (23.45°C)
    1640995200000000,              // timestamp_us
    0x001,                         // tag_id
    TRUST_VERIFIED,                // trust_level
);

// Convert to bytes for transmission
let bytes = atom.to_bytes();

// Parse received bytes
let received = SemanticAtom::from_bytes(&bytes)?;
println!("Temperature: {:.2}°C", received.get_value());
```

## 🏗️ Architecture

### Memory Layout

The 32-byte structure is carefully designed for efficient processing:

```text
Offset | Size | Field           | Description
-------|------|-----------------|-------------
0      | 4    | entity_id       | Unique entity identifier
4      | 4    | telemetry_info  | Type (16b) + Flags (16b)
8      | 4    | predicate_id    | Semantic validation result
12     | 4    | value_mm        | Measurement value (hundredths)
16     | 8    | timestamp_us    | Unix timestamp in microseconds
24     | 4    | tag_trust       | Tag ID (24b) + Trust (8b)
28     | 4    | pqc_anchor      | PQC signature anchor
```

### Field Encoding

- **entity_id**: 32-bit unique identifier for the data source
- **telemetry_info**: `(telemetry_type << 16) | flags`
- **predicate_id**: 32-bit semantic validation result
- **value_mm**: Fixed-point value (divide by 100.0 for float)
- **timestamp_us**: Unix timestamp with microsecond precision
- **tag_trust**: `(tag_id << 8) | trust_level`
- **pqc_anchor**: Reference to external post-quantum signature

## 🔧 Feature Flags

| Feature | Description | Default |
|---------|-------------|---------|
| `std` | Standard library support | ✅ |
| `serde` | Serialization/deserialization | ❌ |
| `pqc` | Post-quantum cryptography utilities | ❌ |

### Usage Examples

#### Minimal Embedded Usage
```toml
[dependencies]
semantic-atom = { version = "0.1", default-features = false }
```

#### Full Feature Set
```toml
[dependencies]
semantic-atom = { version = "0.1", features = ["serde", "pqc"] }
```

## 📊 Telemetry Types

Predefined telemetry type constants:

```rust
use semantic_atom::*;

// Water level in millimeters
TELEMETRY_WATER_LEVEL_MM

// Temperature in Celsius  
TELEMETRY_TEMPERATURE_C

// Humidity in percentage
TELEMETRY_HUMIDITY_PERCENT

// Atmospheric pressure in Pascals
TELEMETRY_PRESSURE_PA

// Precipitation in millimeters
TELEMETRY_PRECIPITATION_MM
```

## 🎯 Trust Levels

Standardized trust level indicators:

```rust
use semantic_atom::*;

TRUST_RAW         // 0x00 - Unvalidated sensor data
TRUST_VERIFIED    // 0x01 - Cryptographically verified
TRUST_ANOMALY     // 0x02 - Anomaly detected
TRUST_ENTERPRISE  // 0xFF - Enterprise-grade validation
```

## ⚠️ Semantic Validation

Built-in predicate system for data quality:

```rust
use semantic_atom::*;

PREDICATE_NORMAL                    // Normal expected data
PREDICATE_WARNING                   // Warning condition
PREDICATE_CRITICAL                  // Critical condition
PREDICATE_CONSISTENCY              // Historically consistent
PREDICATE_TRIANGULATED             // Multi-source validated
PREDICATE_ANOMALY                  // Anomaly detected
PREDICATE_TREND_STABLE             // Stable trend
PREDICATE_TREND_RISING_FAST        // Rapid increase
PREDICATE_SENSOR_STUCK_WARNING     // Sensor possibly stuck
PREDICATE_API_DATA_INCONSISTENCY   // API vs sensor mismatch
PREDICATE_POTENTIAL_DATA_ANOMALY   // Potential anomaly
```

## 🔐 Post-Quantum Security

When the `pqc` feature is enabled:

```rust
#[cfg(feature = "pqc")]
{
    let atom = SemanticAtom::new(/* ... */);
    
    // Compute SHA-256 hash for PQC anchor
    let hash = atom.compute_hash();
    
    // Set PQC anchor reference
    atom.set_pqc_anchor(0xDEADBEEF);
}
```

## 🏭 Example Implementations

See the `examples/` directory for complete implementations:

- **`entities.rs`** - Example entity ID definitions
- **`usage_example.rs`** - Comprehensive usage demonstration

Run examples:

```bash
cargo run --example usage_example --features "serde,pqc"
```

## 📈 Performance

Target performance characteristics:

| Operation | Target | Notes |
|-----------|--------|-------|
| Serialization | < 100ns | Zero-copy |
| Deserialization | < 100ns | Direct memory access |
| Field access | < 10ns | Inline functions |
| Memory usage | 32 bytes | Fixed size |
| Hash computation | ~1μs | SHA-256 (when enabled) |

## 🌍 Standards Compliance

This implementation follows the 32BSA standard specification:

- **Memory Layout**: Exact 32-byte structure as per spec
- **Field Encoding**: Standardized bit-packing for telemetry and tags
- **Trust Model**: Multi-level trust assessment framework
- **Semantic Validation**: Comprehensive predicate system
- **PQC Integration**: Quantum-resistant security anchors

## 🤝 Contributing

We welcome contributions! Please see our [Contributing Guide](CONTRIBUTING.md) for details.

### Development Setup

```bash
git clone https://github.com/LelloOmwei/omwei-atom.git
cd omwei-atom
cargo test
cargo doc --open
```

### Testing

Run all tests:

```bash
cargo test --all-features
```

Run embedded tests:

```bash
cargo test --no-default-features
```

## 📚 Documentation

- **API Documentation**: [docs.rs/omwei-atom](https://docs.rs/omwei-atom)
- **Repository**: [GitHub - LelloOmwei/omwei-atom](https://github.com/LelloOmwei/omwei-atom)
- **Issues & Support**: [GitHub Issues](https://github.com/LelloOmwei/omwei-atom/issues)

## 🔧 Feature Flags

| Feature | Description | Default |
|---------|-------------|---------|
| `std` | Standard library support | ✅ |
| `serde` | Serialization/deserialization | ❌ |
| `pqc` | Post-quantum cryptography utilities | ❌ |

### Usage Examples

#### Minimal Embedded Usage
```toml
[dependencies]
omwei-atom = { version = "0.1", default-features = false }
```

#### Full Feature Set
```toml
[dependencies]
omwei-atom = { version = "0.1", features = ["serde", "pqc"] }
```

## 🎯 Use Cases

### Industrial AI & Automation
- **Predictive Maintenance**: Semantic atoms enable AI agents to understand equipment health
- **Quality Control**: Standardized quality metrics across manufacturing lines
- **Supply Chain Optimization**: Real-time semantic tracking of goods and materials

### Smart Infrastructure
- **Building Management**: Unified semantic data from HVAC, lighting, and security systems
- **Energy Grids**: Semantic understanding of power flow and demand patterns
- **Transportation**: Cross-modal semantic data for traffic and logistics optimization

### Environmental Monitoring
- **Climate Networks**: Semantic interoperability between weather stations and sensors
- **Water Management**: Unified semantic data for hydrological systems
- **Agriculture**: Semantic understanding of soil, weather, and crop conditions

## 📊 Performance

| Operation | Target | Notes |
|-----------|--------|-------|
| Serialization | < 100ns | Zero-copy |
| Deserialization | < 100ns | Direct memory access |
| Field access | < 10ns | Inline functions |
| Memory usage | 32 bytes | Fixed size |
| Hash computation | ~1μs | SHA-256 (when enabled) |

## 🤝 Contributing

We welcome contributions! Please see our [Contributing Guide](CONTRIBUTING.md) for details.

### Development Setup

```bash
git clone https://github.com/LelloOmwei/omwei-atom.git
cd omwei-atom
cargo test
cargo doc --open
```

### Testing

Run all tests:

```bash
cargo test --all-features
```

Run embedded tests:

```bash
cargo test --no-default-features
```

## 📄 License

This project is licensed under either of:

- **MIT License** - [LICENSE-MIT](LICENSE-MIT)
- **Apache License 2.0** - [LICENSE-APACHE](LICENSE-APACHE)

at your option.

## 🔄 OMWEI/SAMS Industrial Ecosystem

The 32B Semantic Atom is the core trust-hierarchy component of the OMWEI/SAMS Industrial Ecosystem, a high-performance framework for 5ns latency hardware validation.

- **Repository**: [GitHub - LelloOmwei/omwei-atom](https://github.com/LelloOmwei/omwei-atom)
- **Issues & Support**: [GitHub Issues](https://github.com/LelloOmwei/omwei-atom/issues)
- **Community**: Contact via GitHub Issues

## 🔄 Version History

### v0.1.0 (2024-01-01)
- Initial reference implementation
- Core 32-byte Semantic Atom structure
- Full no-std support for embedded systems
- Optional serde and PQC features
- Comprehensive documentation and examples
- AI-native design for agentic systems

---

**Built with ❤️ by the OMWEI Community - Powering the Future of Industrial AI**
