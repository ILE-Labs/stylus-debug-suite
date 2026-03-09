#!/bin/bash
# Integration test for Stylus DAP adapter stateful stepping.

# Build it first
cargo build -p debug-adapter --bin stylus-dap

# Helper to send a JSON request and read a line
echo '{"seq": 1, "type": "request", "command": "initialize", "arguments": {}}' > dap_in.txt
echo '{"seq": 2, "type": "request", "command": "launch", "arguments": {"contractPath": "vault.rs"}}' >> dap_in.txt
echo '{"seq": 3, "type": "request", "command": "stackTrace", "arguments": {}}' >> dap_in.txt
echo '{"seq": 4, "type": "request", "command": "next", "arguments": {}}' >> dap_in.txt
echo '{"seq": 5, "type": "request", "command": "stackTrace", "arguments": {}}' >> dap_in.txt
echo '{"seq": 6, "type": "request", "command": "scopes", "arguments": {}}' >> dap_in.txt
echo '{"seq": 7, "type": "request", "command": "variables", "arguments": {"variablesReference": 1001}}' >> dap_in.txt
echo '{"seq": 8, "type": "request", "command": "disconnect", "arguments": {}}' >> dap_in.txt

cat dap_in.txt | ../target/debug/stylus-dap > dap_out.txt

echo "DAP Output Analysis:"
echo "-------------------"
# Check if lines in stackTrace changed
L1=$(grep stackFrames dap_out.txt | sed -n '1p' | grep -o '"line":[0-9]*')
L2=$(grep stackFrames dap_out.txt | sed -n '2p' | grep -o '"line":[0-9]*')

echo "Step 1 line: $L1"
echo "Step 2 line: $L2"

if [ "$L1" != "$L2" ]; then
    echo "SUCCESS: Program counter advanced correctly."
else
    echo "FAILURE: Program counter stuck at $L1."
    exit 1
fi

# Check if variables are returned
VARS=$(grep variables dap_out.txt | grep "stack\[0\]")
if [ -n "$VARS" ]; then
    echo "SUCCESS: Real variables found in stack: $VARS"
else
    echo "FAILURE: No variables found in stack output."
    exit 1
fi

# rm dap_in.txt dap_out.txt
