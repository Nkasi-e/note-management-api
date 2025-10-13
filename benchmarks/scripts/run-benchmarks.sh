#!/bin/bash

# Master benchmark runner script
# Runs all benchmarks and generates reports

set -e

# Get the directory where this script is located
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BENCHMARK_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
PROJECT_ROOT="$(cd "$BENCHMARK_DIR/.." && pwd)"

BASE_URL="${BASE_URL:-http://localhost:3001}"
DURATION="${DURATION:-30s}"
CONNECTIONS="${CONNECTIONS:-100}"
THREADS="${THREADS:-4}"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}"
cat << "EOF"
╔══════════════════════════════════════════════════════════════════════════════╗
║                    🚀 BENCHMARK SUITE RUNNER 🚀                              ║
╚══════════════════════════════════════════════════════════════════════════════╝
EOF
echo -e "${NC}"

# Check if server is running
echo -e "${YELLOW}Checking if server is running...${NC}"
if ! curl -s "$BASE_URL/health" > /dev/null; then
    echo -e "${RED}✗ Server is not running at $BASE_URL${NC}"
    echo -e "${YELLOW}Please start the server first:${NC}"
    echo "  cargo run --release"
    exit 1
fi
echo -e "${GREEN}✓ Server is running${NC}"
echo ""

# Check for wrk
HAS_WRK=false
if command -v wrk &> /dev/null; then
    HAS_WRK=true
    echo -e "${GREEN}✓ wrk found${NC}"
else
    echo -e "${YELLOW}⚠ wrk not found (install: brew install wrk on macOS)${NC}"
fi

# Check for k6
HAS_K6=false
if command -v k6 &> /dev/null; then
    HAS_K6=true
    echo -e "${GREEN}✓ k6 found${NC}"
else
    echo -e "${YELLOW}⚠ k6 not found (install: brew install k6 on macOS)${NC}"
fi

echo ""

# Create results directory
RESULTS_DIR="$BENCHMARK_DIR/results/$(date +%Y%m%d_%H%M%S)"
mkdir -p "$RESULTS_DIR"

echo -e "${BLUE}Results will be saved to: $RESULTS_DIR${NC}"
echo ""

# Get auth token
echo -e "${YELLOW}Getting authentication token...${NC}"
AUTH_RESPONSE=$(curl -s -X POST "$BASE_URL/api/v1/auth/login" \
    -H "Content-Type: application/json" \
    -d '{
        "email": "bench-user-1@example.com",
        "password": "SecurePass123!"
    }')

AUTH_TOKEN=$(echo $AUTH_RESPONSE | grep -o '"token":"[^"]*' | cut -d'"' -f4)

if [ -z "$AUTH_TOKEN" ]; then
    echo -e "${RED}✗ Failed to get auth token. Run seed-data.sh first.${NC}"
    exit 1
fi
echo -e "${GREEN}✓ Authentication successful${NC}"
echo ""

export AUTH_TOKEN

# ============================================================================
# wrk BENCHMARKS
# ============================================================================

if [ "$HAS_WRK" = true ]; then
    echo -e "${BLUE}════════════════════════════════════════${NC}"
    echo -e "${BLUE}   Running wrk Benchmarks${NC}"
    echo -e "${BLUE}════════════════════════════════════════${NC}"
    echo ""
    
    # Health check
    echo -e "${YELLOW}1. Health Check Endpoint${NC}"
    wrk -t$THREADS -c$CONNECTIONS -d$DURATION \
        "$BASE_URL/health" \
        | tee "$RESULTS_DIR/wrk-health.txt"
    echo ""
    
    # Task listing (tests query building)
    echo -e "${YELLOW}2. Task Listing (Query Building)${NC}"
    wrk -t$THREADS -c$CONNECTIONS -d$DURATION \
        -H "Authorization: Bearer $AUTH_TOKEN" \
        -s "$BENCHMARK_DIR/wrk/tasks.lua" \
        "$BASE_URL" \
        | tee "$RESULTS_DIR/wrk-tasks.txt"
    echo ""
    
    # File upload (tests zero-copy)
    echo -e "${YELLOW}3. File Upload (Zero-Copy)${NC}"
    wrk -t$THREADS -c$CONNECTIONS -d$DURATION \
        -s "$BENCHMARK_DIR/wrk/file-upload.lua" \
        "$BASE_URL" \
        | tee "$RESULTS_DIR/wrk-file-upload.txt"
    echo ""
    
    echo -e "${GREEN}✓ wrk benchmarks complete${NC}"
    echo ""
fi

# ============================================================================
# k6 BENCHMARKS
# ============================================================================

if [ "$HAS_K6" = true ]; then
    echo -e "${BLUE}════════════════════════════════════════${NC}"
    echo -e "${BLUE}   Running k6 Benchmarks${NC}"
    echo -e "${BLUE}════════════════════════════════════════${NC}"
    echo ""
    
    # Load test
    echo -e "${YELLOW}1. Load Test (Comprehensive)${NC}"
    k6 run \
        --out json="$RESULTS_DIR/k6-load-test.json" \
        -e BASE_URL="$BASE_URL" \
        "$BENCHMARK_DIR/k6/load-test.js" \
        | tee "$RESULTS_DIR/k6-load-test.txt"
    echo ""
    
    # WebSocket test (tests arena allocation)
    echo -e "${YELLOW}2. WebSocket Test (Arena Allocation)${NC}"
    WS_URL_DERIVED=$(echo "$BASE_URL" | sed 's/http/ws/')
    k6 run \
        --out json="$RESULTS_DIR/k6-websocket.json" \
        -e WS_URL="$WS_URL_DERIVED" \
        "$BENCHMARK_DIR/k6/websocket-test.js" \
        | tee "$RESULTS_DIR/k6-websocket.txt"
    echo ""
    
    echo -e "${GREEN}✓ k6 benchmarks complete${NC}"
    echo ""
fi

# ============================================================================
# SUMMARY
# ============================================================================

echo -e "${BLUE}"
cat << "EOF"
╔══════════════════════════════════════════════════════════════════════════════╗
║                         📊 BENCHMARK COMPLETE 📊                             ║
╚══════════════════════════════════════════════════════════════════════════════╝
EOF
echo -e "${NC}"

echo -e "${GREEN}Results saved to: $RESULTS_DIR${NC}"
echo ""
echo -e "${YELLOW}What was tested:${NC}"
echo "  ✓ Health check endpoint"
echo "  ✓ Task listing with pagination (query building)"
echo "  ✓ File upload (zero-copy optimization)"
echo "  ✓ WebSocket connections (arena allocation)"
echo "  ✓ Complete user scenarios (k6 load test)"
echo ""
echo -e "${YELLOW}Next steps:${NC}"
echo "  1. Review results in $RESULTS_DIR"
echo "  2. Compare with previous benchmarks"
echo "  3. Identify bottlenecks"
echo "  4. Run stress test: k6 run benchmarks/k6/stress-test.js"
echo ""

# Generate quick summary
echo -e "${BLUE}Quick Summary:${NC}"
if [ "$HAS_WRK" = true ]; then
    echo ""
    echo -e "${YELLOW}wrk - Task Listing:${NC}"
    grep "Requests/sec:" "$RESULTS_DIR/wrk-tasks.txt" || true
    grep "Latency" "$RESULTS_DIR/wrk-tasks.txt" || true
fi

if [ "$HAS_K6" = true ]; then
    echo ""
    echo -e "${YELLOW}k6 - Load Test:${NC}"
    grep "http_req_duration" "$RESULTS_DIR/k6-load-test.txt" || true
    grep "http_req_failed" "$RESULTS_DIR/k6-load-test.txt" || true
fi

echo ""
echo -e "${GREEN}✨ All benchmarks complete! ✨${NC}"

