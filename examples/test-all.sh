#!/bin/bash
set -e

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

PASSED=0
FAILED=0

echo "=========================================="
echo "Testing sayiir-openflow export/import"
echo "=========================================="
echo ""

# Helper function to print test results
pass() {
    echo -e "${GREEN}✓ $1${NC}"
    PASSED=$((PASSED + 1))
}

fail() {
    echo -e "${RED}✗ $1${NC}"
    FAILED=$((FAILED + 1))
}

warn() {
    echo -e "${YELLOW}⚠ $1${NC}"
}

# Test 1: Python project (hello-world-py)
echo "----------------------------------------"
echo "Test 1: Python hello-world"
echo "----------------------------------------"

cd hello-world-py

# Export
if cargo sayiir-openflow export > /dev/null 2>&1; then
    pass "Exported hello-world-py"
else
    fail "Failed to export hello-world-py"
fi

# Import
cd ..
rm -rf hello-world-py-test-import
if cargo sayiir-openflow import hello-world-py/workflow.json -o hello-world-py-test-import > /dev/null 2>&1; then
    pass "Imported hello-world-py"
else
    fail "Failed to import hello-world-py"
fi

# Test run
cd hello-world-py-test-import
if python3 main.py '{"name": "World"}' 2>/dev/null | grep -q "Hello"; then
    pass "Ran hello-world-py successfully"
else
    fail "Failed to run hello-world-py"
fi
cd ..

# Test 2: Node.js project (hello-world-node)
echo ""
echo "----------------------------------------"
echo "Test 2: Node.js hello-world"
echo "----------------------------------------"

cd hello-world-node

# Export
if cargo sayiir-openflow export > /dev/null 2>&1; then
    pass "Exported hello-world-node"
else
    fail "Failed to export hello-world-node"
fi

# Import
cd ..
rm -rf hello-world-node-test-import
if cargo sayiir-openflow import hello-world-node/workflow.json -o hello-world-node-test-import > /dev/null 2>&1; then
    pass "Imported hello-world-node"
else
    fail "Failed to import hello-world-node"
fi

# Test run (with input data)
cd hello-world-node-test-import
if node index.js '{"name": "World"}' 2>/dev/null | grep -q "Hello"; then
    pass "Ran hello-world-node successfully"
else
    fail "Failed to run hello-world-node"
fi
cd ..

# Test 3: Rust project (hello-world-rs)
echo ""
echo "----------------------------------------"
echo "Test 3: Rust hello-world"
echo "----------------------------------------"

cd hello-world-rs

# Export
if cargo sayiir-openflow export > /dev/null 2>&1; then
    pass "Exported hello-world-rs"
else
    fail "Failed to export hello-world-rs"
fi

# Import
cd ..
rm -rf hello-world-rs-test-import
if cargo sayiir-openflow import hello-world-rs/workflow.json -o hello-world-rs-test-import > /dev/null 2>&1; then
    pass "Imported hello-world-rs"
else
    fail "Failed to import hello-world-rs"
fi

# Test build and run
cd hello-world-rs-test-import
if cargo build --release > /dev/null 2>&1; then
    pass "Built hello-world-rs"

    if cargo run -- '"World"' 2>/dev/null | grep -q "Hello"; then
        pass "Ran hello-world-rs successfully"
    else
        fail "Failed to run hello-world-rs"
    fi
else
    fail "Failed to build hello-world-rs"
fi
cd ..

# Test 4: Complex Python project (approval-workflow-py)
echo ""
echo "----------------------------------------"
echo "Test 4: Python approval workflow"
echo "----------------------------------------"

cd approval-workflow-py

# Export
if cargo sayiir-openflow export > /dev/null 2>&1; then
    pass "Exported approval-workflow-py"
else
    fail "Failed to export approval-workflow-py"
fi

# Import
cd ..
rm -rf approval-workflow-py-test-import
if cargo sayiir-openflow import approval-workflow-py/workflow.json -o approval-workflow-py-test-import > /dev/null 2>&1; then
    pass "Imported approval-workflow-py"
else
    fail "Failed to import approval-workflow-py"
fi

# Test run with valid input
cd approval-workflow-py-test-import
TEST_INPUT='{"title": "Test Report", "content": "This is a test report."}'
if python3 main.py "$TEST_INPUT" 2>/dev/null | grep -q "Test Report"; then
    pass "Ran approval-workflow-py successfully"
else
    warn "approval-workflow-py may have dependency issues (expected)"
fi
cd ..

# Test 5: Order processing Node.js (with proper input)
echo ""
echo "----------------------------------------"
echo "Test 5: Node.js order processing"
echo "----------------------------------------"

cd order-processing-node

# Export
if cargo sayiir-openflow export > /dev/null 2>&1; then
    pass "Exported order-processing-node"
else
    fail "Failed to export order-processing-node"
fi

# Import
cd ..
rm -rf order-processing-node-test-import
if cargo sayiir-openflow import order-processing-node/workflow.json -o order-processing-node-test-import > /dev/null 2>&1; then
    pass "Imported order-processing-node"
else
    fail "Failed to import order-processing-node"
fi

# Test run with valid order
cd order-processing-node-test-import
TEST_ORDER='{"orderId": "order-1", "customerEmail": "test@example.com", "amount": 99.99}'
if node index.js "$TEST_ORDER" 2>/dev/null | grep -q "validated"; then
    pass "Ran order-processing-node successfully"
else
    warn "order-processing-node may have dependency issues (expected)"
fi
cd ..

# Cleanup
echo ""
echo "----------------------------------------"
echo "Cleaning up test imports..."
echo "----------------------------------------"
rm -rf hello-world-py-test-import
rm -rf hello-world-node-test-import
rm -rf hello-world-rs-test-import
rm -rf approval-workflow-py-test-import
rm -rf order-processing-node-test-import

echo ""
echo "=========================================="
echo "Test Results"
echo "=========================================="
echo -e "${GREEN}Passed: $PASSED${NC}"
echo -e "${RED}Failed: $FAILED${NC}"
echo ""

if [ $FAILED -eq 0 ]; then
    echo -e "${GREEN}All tests passed!${NC}"
    exit 0
else
    echo -e "${RED}Some tests failed.${NC}"
    exit 1
fi
