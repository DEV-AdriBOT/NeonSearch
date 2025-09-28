#!/bin/bash

# NeonSearch Run Script  
# Runs the pre-built NeonSearch browser for faster startup

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo "🚀 Starting NeonSearch Browser by NeonDev™"
echo "==========================================="

# Get project root directory
PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$PROJECT_ROOT"

# Check if release binary exists
BINARY_PATH=""
if [[ "$OSTYPE" == "darwin"* ]]; then
    if [[ -f "target/release/neonsearch" ]]; then
        BINARY_PATH="target/release/neonsearch"
    elif [[ -f "dist/NeonSearch" ]]; then
        BINARY_PATH="dist/NeonSearch"
    fi
elif [[ -f "target/release/neonsearch" ]]; then
    BINARY_PATH="target/release/neonsearch"
elif [[ -f "dist/neonsearch" ]]; then
    BINARY_PATH="dist/neonsearch"
fi

if [[ -z "$BINARY_PATH" ]]; then
    echo -e "${RED}❌ NeonSearch binary not found!${NC}"
    echo -e "${YELLOW}💡 Please build first with: ./build.sh${NC}"
    echo
    echo -e "${BLUE}🔨 Building now...${NC}"
    if cargo build --release; then
        echo -e "${GREEN}✅ Build successful!${NC}"
        if [[ "$OSTYPE" == "darwin"* ]]; then
            BINARY_PATH="target/release/neonsearch"
        else
            BINARY_PATH="target/release/neonsearch"
        fi
    else
        echo -e "${RED}❌ Build failed${NC}"
        exit 1
    fi
fi

echo -e "${GREEN}✅ Found NeonSearch at: $BINARY_PATH${NC}"
echo -e "${BLUE}🌐 Launching NeonSearch Browser...${NC}"
echo

# Run the browser
"$BINARY_PATH"