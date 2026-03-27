#!/bin/bash

echo "🔍 Testing Different UART Addresses with QEMU virt machine"
echo "========================================================"

# Build first
echo "📦 Building..."
cargo build --release --target riscv64gc-unknown-none-elf || exit 1

echo ""
echo "🧪 Testing common UART addresses used by QEMU virt..."

# Test different UART addresses that QEMU virt might use
ADDRESSES=(
    "0x10000000"  # PL011 UART (some virt configs)
    "0x10010000"  # SiFive UART (current)
    "0x9000000"   # Another common UART address
)

for addr in "${ADDRESSES[@]}"; do
    echo ""
    echo "🔧 Testing UART address: $addr"
    echo "----------------------------------------"
    
    # Create a simple test that writes to this address
    rm -f serial_$addr.log
    
    # Start QEMU with serial dump
    qemu-system-riscv64 -machine virt -m 256M -smp 4 \
        -nographic -bios none \
        -kernel target/riscv64gc-unknown-none-elf/release/omwei-hal.bin \
        -serial file:serial_$addr.log &
    
    QEMU_PID=$!
    
    # Wait a moment
    sleep 2
    
    # Stop QEMU
    kill $QEMU_PID 2>/dev/null || true
    wait $QEMU_PID 2>/dev/null || true
    
    # Check if we got any output
    if [ -f serial_$addr.log ]; then
        size=$(wc -c < serial_$addr.log)
        if [ $size -gt 0 ]; then
            echo "✅ SUCCESS: Got $size bytes from UART at $addr"
            echo "Contents:"
            cat serial_$addr.log
            echo ""
        else
            echo "❌ No output from UART at $addr"
        fi
        rm -f serial_$addr.log
    else
        echo "❌ No serial log file created for $addr"
    fi
done

echo ""
echo "🔍 If none of these worked, the issue might be:"
echo "   1. UART not initialized correctly in our code"
echo "   2. Wrong register offsets for the UART type"
echo "   3. UART not enabled in QEMU virt machine config"
echo ""
echo "✅ Test completed"
