#!/bin/bash

echo "🔧 Serial Dump Test - Force QEMU to write UART to file"
echo "======================================================"

# Clean up any existing log file
rm -f serial.log

# Build first
echo "📦 Building..."
cargo build --release --target riscv64gc-unknown-none-elf || exit 1

echo ""
echo "🚀 Starting QEMU with serial dump to file..."
echo "   UART output will be written to: serial.log"
echo "   We'll tail the file to see output in real-time"
echo ""

# Start QEMU with serial output to file
qemu-system-riscv64 -machine virt -m 256M -smp 4 \
    -nographic -bios none \
    -kernel target/riscv64gc-unknown-none-elf/release/omwei-hal.bin \
    -serial file:serial.log &

QEMU_PID=$!

echo "QEMU started with PID: $QEMU_PID"
echo ""

# Wait a moment for QEMU to start and potentially write to the file
sleep 1

# Check if the file exists and show its contents
if [ -f serial.log ]; then
    echo "📄 Serial log file contents:"
    echo "--------------------------------"
    cat serial.log
    echo "--------------------------------"
    echo ""
    
    # Start tailing the file to see real-time output
    echo "👀 Tailing serial.log for real-time output (Ctrl+C to stop):"
    tail -f serial.log &
    TAIL_PID=$!
    
    # Wait for user to stop or timeout
    sleep 10
    
    # Clean up
    kill $TAIL_PID 2>/dev/null || true
else
    echo "❌ serial.log file was not created"
    echo "   This suggests QEMU may not be writing to UART at all"
fi

# Stop QEMU
echo ""
echo "🛑 Stopping QEMU..."
kill $QEMU_PID 2>/dev/null || true
wait $QEMU_PID 2>/dev/null || true

# Show final file contents
echo ""
echo "📋 Final serial.log contents:"
echo "================================"
if [ -f serial.log ]; then
    cat serial.log
    echo ""
    echo "📊 File size: $(wc -c < serial.log) bytes"
else
    echo "No serial.log file found"
fi

echo "✅ Test completed"
