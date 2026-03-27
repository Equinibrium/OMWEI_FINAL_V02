#!/bin/bash

# Quick test to capture initial output
echo "🧪 Testing UART output..."
./run_sim.sh > test_output.txt 2>&1 &
PID=$!

# Wait 5 seconds for initial output
sleep 5

# Kill the process
kill $PID 2>/dev/null || true
wait $PID 2>/dev/null || true

# Show the output
echo "📋 Full output captured:"
cat test_output.txt

# Look for our debug markers and text
echo ""
echo "🔍 Looking for debug markers and text:"
echo "Debug markers found:"
grep -o "[CRTASPH]" test_output.txt | sort | uniq -c

echo ""
echo "Looking for 'Hello' text:"
grep -i "hello" test_output.txt || echo "No 'Hello' found"

echo ""
echo "Looking for any printable text (excluding build output):"
grep -E "^[A-Za-z0-9!@#$%^&*()_+\-=\[\]{};':\"\\|,.<>/?\s]+$" test_output.txt | head -5 || echo "No printable text found"

# Clean up
rm -f test_output.txt
