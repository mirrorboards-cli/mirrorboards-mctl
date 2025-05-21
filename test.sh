#!/bin/bash
# Simple test script for mctl

set -e

echo "Testing mctl tool..."

# Create a test directory
TEST_DIR="test_mctl"
mkdir -p $TEST_DIR
cd $TEST_DIR

# Create a test mirror.toml file
cat > mirror.toml << EOF
[[repositories]]
origin = "https://github.com/rust-lang/rust-analyzer.git"
path = "rust-analyzer"

[[repositories]]
origin = "https://github.com/rust-lang/cargo.git"
path = "cargo"
branch = "master"
EOF

echo "Created test mirror.toml file"

# Run mctl sync
echo "Running mctl sync..."
../target/release/mctl sync --verbose

# Verify that the repositories were cloned
if [ -d "rust-analyzer" ] && [ -d "cargo" ]; then
    echo "✅ Test passed! Repositories were successfully cloned."
else
    echo "❌ Test failed! Repositories were not cloned correctly."
    exit 1
fi

# Clean up
cd ..
echo "Cleaning up test directory..."
# rm -rf $TEST_DIR

echo "Test completed successfully!"