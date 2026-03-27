#!/bin/bash

# Simple test - run QEMU and capture output immediately
echo "🧪 Simple UART test..."

# Build first
cargo build --release --target riscv64gc-unknown-none-elf

# Run QEMU with direct output capture
timeout 3s qemu-system-riscv64 \
  -machine virt \
  -cpu rv64 \
  -smp 4 \
  -m 128M \
  -nographic \
  -serial stdio \
  -bios none \
  -kernel target/riscv64gc-unknown-none-elf/release/omwei-hal \
  -d guest_errors,unimp 2>&1 || echo "QEMU terminated"

echo ""
echo "✅ Test completed"
