#!/bin/bash

# NeonSearch Cross-Platform Run Script  
# Runs the pre-built NeonSearch browser for faster startup

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
PURPLE='\033[0;35m'
NC='\033[0m' # No Color

echo "🚀 Starting NeonSearch Browser by NeonDev™"
echo "==========================================="

# Detect OS
detect_os() {
    if [[ "$OSTYPE" == "darwin"* ]]; then
        echo "macos"
    elif [[ "$OSTYPE" == "linux-gnu"* ]]; then
        echo "linux"
    elif [[ "$OSTYPE" == "msys" || "$OSTYPE" == "cygwin" ]]; then
        echo "windows"
    else
        echo "unknown"
    fi
}

OS=$(detect_os)
echo -e "${BLUE}📍 Detected OS: ${CYAN}$OS${NC}"

# Get project root directory
PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$PROJECT_ROOT"

# OS-specific binary configuration
configure_binary() {
    case $OS in
        "macos")
            BINARY_NAME="NeonSearch"
            BINARY_EXT=""
            ;;
        "linux")
            BINARY_NAME="neonsearch"
            BINARY_EXT=""
            ;;
        "windows")
            BINARY_NAME="neonsearch"
            BINARY_EXT=".exe"
            ;;
        *)
            BINARY_NAME="neonsearch"
            BINARY_EXT=""
            ;;
    esac
}

configure_binary

# Check if release binary exists
BINARY_PATH=""

# Look for binary in multiple locations
BINARY_LOCATIONS=(
    "dist/${BINARY_NAME}${BINARY_EXT}"
    "target/release/neonsearch${BINARY_EXT}"
)

for location in "${BINARY_LOCATIONS[@]}"; do
    if [[ -f "$location" ]]; then
        BINARY_PATH="$location"
        break
    fi
done

if [[ -z "$BINARY_PATH" ]]; then
    echo -e "${RED}❌ NeonSearch binary not found!${NC}"
    echo -e "${YELLOW}💡 Please build first with: ./build.sh${NC}"
    echo
    echo -e "${BLUE}🔨 Building now...${NC}"
    
    # Use appropriate build script for OS
    if [[ "$OS" == "windows" ]]; then
        BUILD_CMD="build.bat"
    else
        BUILD_CMD="./build.sh"
    fi
    
    if $BUILD_CMD; then
        echo -e "${GREEN}✅ Build successful!${NC}"
        
        # Try to find the binary again
        for location in "${BINARY_LOCATIONS[@]}"; do
            if [[ -f "$location" ]]; then
                BINARY_PATH="$location"
                break
            fi
        done
        
        if [[ -z "$BINARY_PATH" ]]; then
            echo -e "${RED}❌ Binary still not found after build${NC}"
            exit 1
        fi
    else
        echo -e "${RED}❌ Build failed${NC}"
        exit 1
    fi
fi

echo -e "${GREEN}✅ Found NeonSearch at: $BINARY_PATH${NC}"

# Set OS-specific environment variables
case $OS in
    "macos")
        export OBJC_DISABLE_INITIALIZE_FORK_SAFETY=YES
        ;;
    "linux")
        # Linux-specific optimizations if needed
        ;;
    "windows")
        # Windows-specific settings if needed
        ;;
esac

echo -e "${BLUE}🌐 Launching NeonSearch Browser for $OS...${NC}"
echo

# Run the browser
"$BINARY_PATH"