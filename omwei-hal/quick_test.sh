#!/bin/bash

echo "🧪 Quick UART Test with Corrected Settings"
echo "=========================================="

# Build first
echo "📦 Building..."
cargo build --release --target riscv64gc-unknown-none-elf || exit 1

echo ""
echo "🔧 Running QEMU with UART at 0x10000000 - should see each character on new line:"
echo "Expected output:"
echo "C"
echo "T" 
echo "E"
echo "S"
echo "T"
echo "P"
echo "R"
echo "T" 
echo "A"
echo "S"
echo ""

# Run QEMU and capture output
echo "🚀 Starting simulation (will run for 3 seconds)..."
qemu-system-riscv64 -machine virt -cpu rv64 -smp 4 -m 128M -nographic -serial mon:stdio -bios none -kernel target/riscv64gc-unknown-none-elf/release/omwei-hal 2>&1 &
PID=$!

# Wait and capture
sleep 3
kill $PID 2>/dev/null || true
wait $PID 2>/dev/null || true

echo ""
echo "✅ Test completed"
