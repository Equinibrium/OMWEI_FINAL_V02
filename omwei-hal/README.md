# OMWEI Equinibrium SoC - Semantic Navigator Implementation

## 🎯 Overview

This project implements a high-speed **Semantic Navigator** for parallel graph traversal on the OMWEI Equinibrium SoC. The implementation leverages the SLC (Semantic Link Comparator) hardware accelerator for optimal performance across multiple harts.

## 🚀 Features

### **Semantic Triple Processing**
- **96-byte SemanticTriple struct** with 256-bit aligned subject, predicate, object atoms
- **TripleStore** supporting up to 1024 triples with 32-byte alignment
- **Hardware-optimized** for 128-bit Memory Port access

### **Parallel Search Engine**
- **4-Hart Parallel Processing**: Hart 0 orchestrates, Harts 1-3 as workers
- **SLC Hardware Acceleration**: Single-cycle 256-bit atom comparison
- **IPI Coordination**: Inter-processor interrupts for work distribution
- **DLS Local Storage**: Fast local memory (0x1800_0000) for search results

### **Performance Optimizations**
- **L2 Cache Prefetching**: 48-stream hardware prefetcher warming
- **Batch Processing**: 8-triple batches for pipeline efficiency
- **Memory Barriers**: Proper `fence iorw, iorw` for consistency
- **Atomic Operations**: Lock-free counters for scalability

### **UART0 Console Output**
- **115200 baud** serial communication at 0x10010000
- **Real-time logging** of verification results and performance metrics
- **SLC Integration Success** message confirmation

## 🛠 Build & Run

### Prerequisites
```bash
# Install Rust target
rustup target add riscv64gc-unknown-none-elf

# Install LLVM tools
rustup component add llvm-tools

# Install QEMU (for simulation)
# macOS:
brew install qemu
# Ubuntu:
sudo apt-get install qemu-system-riscv
```

### Build & Simulation
```bash
# Build and run simulation
./run_sim.sh

# Build and run with GDB debugging
./run_sim.sh --gdb

# Manual build
cargo build --release --target riscv64gc-unknown-none-elf
rust-objcopy -O binary target/riscv64gc-unknown-none-elf/release/omwei-hal \
    target/riscv64gc-unknown-none-elf/release/omwei-hal.bin
```

## 📊 Verification Results

The Semantic Navigator verification performs:

1. **300 Random Triple Generation** using TRNG hardware
2. **5 Target Predicate Insertion** (every 60th triple)
3. **Parallel Search** across 4 harts with SLC acceleration
4. **Performance Measurement** using Trace Timestamp

### Expected Output
```
🚀 OMWEI Equinibrium SoC - Hart 0 Starting
📡 UART0 initialized at 0x10010000
🔧 Performing SLC hardware vs software verification...
✅ SLC INTEGRATION SUCCESS - Hardware acceleration working!
🧭 Starting Semantic Navigator verification...
📊 Creating 300 random semantic triples...
🔍 Starting parallel search for target predicate...
📈 Search completed in XXXX cycles
🎯 Found 5 matches (expected: 5)
⚡ Search rate: XXX triples/1000 cycles
✅ Semantic Navigator verification completed!
```

## 🏗 Architecture

### **Memory Layout**
- **RAM**: 0x8000_0000 - Triple storage
- **SLC**: 0x7000_0000 - Hardware comparator
- **DLS**: 0x1800_0000 - Local result storage
- **UART0**: 0x1001_0000 - Serial output
- **Work Assignments**: 0x9000_0000 - Per-hart work distribution

### **Hardware Acceleration**
- **SLC Base**: 0x7000_0000 for all predicate comparisons
- **128-bit Port**: Four 64-bit writes with memory fences
- **Single Cycle**: 256-bit atom comparison capability

### **Parallel Execution**
```
Hart 0: Orchestrator
├── Distributes work (triples/3 per hart)
├── Sends IPIs to Harts 1-3
├── Waits for completion
└── Collects and reports results

Harts 1-3: Workers
├── Wait for IPI and work assignment
├── Prefetch L2 cache range
├── Search using SLC acceleration
├── Store matches in DLS
└── Signal completion
```

## 🔧 Configuration

### **Target Configuration**
- **CPU**: RISC-V 64-bit GC ISA
- **Harts**: 4 hardware threads
- **Memory**: 512MB RAM
- **Machine**: SiFive-U (QEMU compatible)

### **Performance Tuning**
- **Batch Size**: 8 triples for optimal pipeline
- **Prefetch Depth**: 48 L2 cache streams
- **Work Distribution**: Equal segments across harts
- **Result Buffer**: 256 entries in DLS

## 🐛 Debugging

### **GDB Debugging**
```bash
# Run with GDB server
./run_sim.sh --gdb

# Connect GDB
riscv64-unknown-elf-gdb target/riscv64gc-unknown-none-elf/release/omwei-hal
(gdb) target remote localhost:1234
(gdb) break hart0_main
(gdb) continue
```

### **UART Output**
All console output is sent to UART0 (0x10010000) and appears in the terminal when running with QEMU.

## 📈 Performance Metrics

The implementation tracks:
- **Triples Searched**: Total number processed
- **Matches Found**: Count of predicate matches
- **Search Time**: Cycles measured with Trace Timestamp
- **Search Rate**: Triples per 1000 cycles

## 🔄 Development

### **Project Structure**
```
src/
├── main.rs              # Main application and verification
├── lib.rs               # Library exports
├── uart.rs              # UART0 driver for console output
├── semantic/
│   ├── mod.rs           # Semantic data structures
│   ├── navigator.rs     # Parallel search implementation
│   └── scie.rs          # SLC hardware interface
├── crypto/
│   └── trng.rs         # True Random Number Generator
├── interrupts/         # IPI and APLIC interrupt handling
├── registers/           # SoC register definitions
└── sync.rs              # Synchronization primitives
```

### **Adding New Features**
1. **Semantic Operations**: Extend `semantic/mod.rs`
2. **Hardware Interfaces**: Add to `registers/mod.rs`
3. **Parallel Algorithms**: Extend `semantic/navigator.rs`
4. **Console Output**: Use `print!()` and `println!()` macros

## 📜 License

This project is part of the OMWEI Equinibrium SoC implementation.

---

**🎯 Mission**: Achieve maximum performance for semantic graph traversal through hardware acceleration and parallel processing.

### Memory Map
| Region | Base Address | Size | Description |
|--------|--------------|------|-------------|
| FLASH | 0x2000_0000 | 512MB | Peripheral Port Flash |
| RAM | 0x8000_0000 | 512MB | 128-bit Memory Port RAM |
| CLINT | 0x0200_0000 | - | Timer/IPI Controller |
| PMC | 0x0310_0000 | - | Power Management Controller |
| HCA | 0x051A_0000 | - | Hardware Crypto Accelerator |
| APLIC | 0x0C00_0000 | - | Advanced Interrupt Controller |
| ITIM | 0x0180_0000 | 16KB | Instruction Tightly Integrated Memory |
| DLS | 0x1800_0000 | 64KB | Data Local Storage |

## Features

- **Multi-core Support**: 4-hart boot with parking mechanism
- **Memory Management**: Custom memory regions for FLASH, RAM, ITIM, and DLS
- **Interrupt Handling**: APLIC and CLINT register definitions
- **No-STD Environment**: Fully embedded, no standard library
- **Professional HAL**: Clean abstraction for hardware access

## Project Structure

```
omwei-hal/
├── .cargo/
│   └── config.toml          # RISC-V target configuration
├── src/
│   ├── boot.S              # Multi-core boot code
│   ├── main.rs             # Main kernel entry point
│   ├── lib.rs              # Library interface
│   └── registers/
│       └── mod.rs          # Hardware register definitions
├── Cargo.toml              # Project dependencies
├── memory.x                # Memory region definitions
├── link.x                  # Linker script
└── README.md               # This file
```

## Building

### Prerequisites
- Rust with RISC-V target: `rustup target add riscv64gc-unknown-none-elf`
- RISC-V toolchain for debugging: `riscv64-unknown-elf-gdb`

### Build Commands
```bash
# Build the project
cargo build

# Build for release
cargo build --release

# Run with GDB (requires RISC-V GDB)
cargo run
```

## Usage

The HAL provides a clean interface for accessing OMWEI Equinibrium SoC features:

```rust
use omwei_hal::registers;

// Access CLINT registers
let mtimecmp = registers::clint::MTIMECMP;

// Access APLIC registers
let domaincfg = registers::aplic::DOMAINCFG;

// Get memory information
let flash_base = registers::memory::FLASH_BASE;
let ram_size = registers::memory::RAM_SIZE;
```

## Multi-Core Boot

The boot sequence handles 4 harts:

1. **Hart 0**: Primary boot hart, initializes system and runs main kernel
2. **Harts 1-3**: Secondary harts, parked using WFI until signaled

Each hart has its own 128KB stack in RAM for safe execution.

## Memory Regions

- **FLASH**: Code storage via peripheral port
- **RAM**: Main memory with 128-bit port for high bandwidth
- **ITIM**: Fast instruction memory for critical code
- **DLS**: Fast data memory for critical data structures

## Interrupt Architecture

- **CLINT**: Local interrupts (timer, inter-processor)
- **APLIC**: Advanced platform-level interrupts (511 inputs, 31 priorities)

## License

MIT OR Apache-2.0
