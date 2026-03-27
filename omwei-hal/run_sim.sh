#!/bin/bash

# OMWEI Equinibrium SoC - QEMU Simulation Script
# Runs the Semantic Navigator with SLC hardware acceleration

set -e

echo "🚀 Building OMWEI Equinibrium SoC..."

# Build in release mode for RISC-V 64-bit
cargo build --release --target riscv64gc-unknown-none-elf

echo "📦 Converting ELF to binary..."

# Convert ELF to binary using rust-objcopy
rust-objcopy -O binary target/riscv64gc-unknown-none-elf/release/omwei-hal \
    target/riscv64gc-unknown-none-elf/release/omwei-hal.bin

echo "🔍 Debugging binary content..."
ls -l target/riscv64gc-unknown-none-elf/release/omwei-hal.bin
echo ""

echo "🔍 Verifying _start symbol in binary..."
# Try different nm commands for macOS
if command -v riscv64-unknown-elf-nm &> /dev/null; then
    riscv64-unknown-elf-nm target/riscv64gc-unknown-none-elf/release/omwei-hal | grep "_start" || echo "❌ _start symbol not found!"
else
    rust-objdump -t target/riscv64gc-unknown-none-elf/release/omwei-hal | grep "_start" || echo "❌ _start symbol not found!"
fi
echo ""

echo "🔍 First 50 bytes of disassembly..."
# Try different objdump commands for macOS
if command -v riscv64-unknown-elf-objdump &> /dev/null; then
    riscv64-unknown-elf-objdump -D target/riscv64gc-unknown-none-elf/release/omwei-hal | head -n 50
else
    rust-objdump --arch-name=riscv64 -D target/riscv64gc-unknown-none-elf/release/omwei-hal | head -n 50
fi

echo "🔧 Launching QEMU with 4 Harts..."

# Check if QEMU is installed
if ! command -v qemu-system-riscv64 &> /dev/null; then
    echo "❌ QEMU not found. Please install QEMU:"
    echo "   macOS: brew install qemu"
    echo "   Ubuntu: sudo apt-get install qemu-system-riscv"
    echo "   Or download from: https://www.qemu.org/download/"
    echo ""
    echo "📁 Binary created at: target/riscv64gc-unknown-none-elf/release/omwei-hal"
    echo "📁 ELF file at: target/riscv64gc-unknown-none-elf/release/omwei-hal.bin"
    exit 1
fi

# Try QEMU with virt machine (UART at 0x10000000) - DISABLE OpenSBI
echo "🔧 Trying QEMU with virt machine (no OpenSBI)..."
QEMU_CMD="qemu-system-riscv64 -machine virt -cpu rv64 -smp 4 -m 128M -nographic -serial mon:stdio -bios none -kernel target/riscv64gc-unknown-none-elf/release/omwei-hal -d guest_errors,unimp"

# Check for GDB flag
if [ "$1" == "--gdb" ]; then
    echo "🐛 Waiting for GDB connection on localhost:1234..."
    QEMU_CMD="$QEMU_CMD -s -S"
    echo "💡 Connect with: riscv64-unknown-elf-gdb target/riscv64gc-unknown-none-elf/release/omwei-hal"
    echo "   Then run: target remote localhost:1234"
fi

echo "📡 UART0 mapped at 0x10000000 - Watch for 'SLC INTEGRATION SUCCESS'"
echo "🏃 Running simulation..."

# Launch QEMU
eval $QEMU_CMD

echo "✅ Simulation completed!"
