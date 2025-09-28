#!/bin/bash

# NeonSearch Build Script
# Builds the NeonSearch browser in release mode

set -e

echo "🔧 Building NeonSearch Browser by NeonDev™"
echo "==========================================="

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Get project root directory
PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$PROJECT_ROOT"

# Check if Rust is installed
if ! command -v cargo &> /dev/null; then
    echo -e "${RED}❌ Rust/Cargo not found. Please install Rust first.${NC}"
    echo "Visit: https://rustup.rs/"
    exit 1
fi

echo -e "${BLUE}📋 Rust version: $(rustc --version)${NC}"
echo -e "${BLUE}📋 Cargo version: $(cargo --version)${NC}"
echo

# Build in release mode
echo -e "${YELLOW}🔨 Building NeonSearch (release mode)...${NC}"

SECONDS=0
cargo build --release &
BUILD_PID=$!

if [ -t 1 ]; then
    # Lightweight spinner so long builds show activity
    SPIN_CHARS='|/-\\'
    spin() {
        local i=0
        while kill -0 "$BUILD_PID" 2>/dev/null; do
            printf "\r${YELLOW}🔨 Building NeonSearch (release mode)... %s${NC}" "${SPIN_CHARS:i++%4:1}"
            sleep 0.2
        done
    }
    spin &
    SPINNER_PID=$!
fi

wait $BUILD_PID
BUILD_STATUS=$?

if [ -n "${SPINNER_PID:-}" ]; then
    kill "$SPINNER_PID" 2>/dev/null || true
    wait "$SPINNER_PID" 2>/dev/null || true
    printf "\r"
fi

ELAPSED=$SECONDS
MINUTES=$((ELAPSED / 60))
SECONDS_REMAINDER=$((ELAPSED % 60))
ELAPSED_FORMATTED=$(printf "%02d:%02d" "$MINUTES" "$SECONDS_REMAINDER")

if [ $BUILD_STATUS -eq 0 ]; then
    echo -e "${GREEN}✅ Release build successful!${NC} (${ELAPSED_FORMATTED})"
else
    echo -e "${RED}❌ Release build failed after ${ELAPSED_FORMATTED}${NC}"
    exit $BUILD_STATUS
fi

# Create dist directory if it doesn't exist
mkdir -p dist

# Copy binary to dist folder for easy access
if [[ "$OSTYPE" == "darwin"* ]]; then
    cp "target/release/neonsearch" "dist/NeonSearch"
    echo -e "${GREEN}📦 Binary copied to dist/NeonSearch${NC}"
elif [[ "$OSTYPE" == "linux"* ]]; then
    cp "target/release/neonsearch" "dist/neonsearch"
    echo -e "${GREEN}📦 Binary copied to dist/neonsearch${NC}"
else
    cp "target/release/neonsearch" "dist/neonsearch"
    echo -e "${GREEN}📦 Binary copied to dist/neonsearch${NC}"
fi

echo
echo -e "${GREEN}🎉 Build completed successfully!${NC}"
echo -e "${BLUE}📍 Run with: ./run.sh${NC}"
echo -e "${BLUE}📍 Or directly: ./dist/NeonSearch${NC}"