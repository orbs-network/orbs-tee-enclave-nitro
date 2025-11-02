#!/bin/bash
# Helper script to test SDK with Docker

set -e

echo "🔍 Checking Docker daemon..."
if ! docker info > /dev/null 2>&1; then
    echo "❌ Docker daemon is not running"
    echo "Please start Docker Desktop and try again"
    exit 1
fi

echo "✅ Docker daemon is running"
echo ""

echo "=========================================="
echo "Testing ORBS TEE Nitro SDK on Linux"
echo "=========================================="
echo ""

# Test 1: Cross-platform tests
echo "1️⃣  Running cross-platform tests (no nitro features)..."
docker build --target test-no-nitro -t orbs-tee-nitro:test-no-nitro .
echo "✅ Cross-platform tests passed"
echo ""

# Test 2: Check nitro features compile
echo "2️⃣  Checking vsock compilation on Linux (with nitro features)..."
docker build --target build-nitro -t orbs-tee-nitro:build-nitro .
echo "✅ vsock compiles on Linux!"
echo ""

# Test 3: Clippy linter
echo "3️⃣  Running clippy linter..."
docker build --target clippy -t orbs-tee-nitro:clippy .
echo "✅ Clippy checks passed"
echo ""

# Test 4: Format check
echo "4️⃣  Checking code formatting..."
docker build --target fmt -t orbs-tee-nitro:fmt .
echo "✅ Format checks passed"
echo ""

echo "=========================================="
echo "✅ All Docker tests passed!"
echo "=========================================="
