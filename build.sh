#!/bin/bash

# NeonSearch Cross-Platform Build Script
# Builds the NeonSearch browser in release mode for macOS, Linux, and Windows

set -e

echo "🔧 Building NeonSearch Browser by NeonDev™"
echo "==========================================="

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
PURPLE='\033[0;35m'
NC='\033[0m' # No Color

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

# Check if Rust is installed
if ! command -v cargo &> /dev/null; then
    echo -e "${RED}❌ Rust/Cargo not found. Please install Rust first.${NC}"
    echo -e "${YELLOW}💡 Run ${CYAN}./dependencies.sh${YELLOW} to install all dependencies${NC}"
    exit 1
fi

echo -e "${BLUE}📋 Rust version: $(rustc --version)${NC}"
echo -e "${BLUE}📋 Cargo version: $(cargo --version)${NC}"

# OS-specific build configurations
configure_build() {
    case $OS in
        "macos")
            echo -e "${CYAN}🍎 Configuring build for macOS...${NC}"
            # macOS-specific environment variables for egui compatibility
            export OBJC_DISABLE_INITIALIZE_FORK_SAFETY=YES
            BUILD_FLAGS="--release"
            BINARY_NAME="NeonSearch"
            BINARY_EXT=""
            ;;
        "linux")
            echo -e "${CYAN}🐧 Configuring build for Linux...${NC}"
            BUILD_FLAGS="--release"
            BINARY_NAME="neonsearch"
            BINARY_EXT=""
            ;;
        "windows")
            echo -e "${CYAN}🪟 Configuring build for Windows...${NC}"
            BUILD_FLAGS="--release"
            BINARY_NAME="neonsearch"
            BINARY_EXT=".exe"
            ;;
        *)
            echo -e "${YELLOW}⚠️  Unknown OS, using default configuration${NC}"
            BUILD_FLAGS="--release"
            BINARY_NAME="neonsearch"
            BINARY_EXT=""
            ;;
    esac
}

configure_build
echo

# Build in release mode
echo -e "${YELLOW}🔨 Building NeonSearch ($OS release mode)...${NC}"

SECONDS=0
cargo build $BUILD_FLAGS &
BUILD_PID=$!

if [ -t 1 ]; then
    # Lightweight spinner so long builds show activity
    SPIN_CHARS='|/-\\'
    spin() {
        local i=0
        while kill -0 "$BUILD_PID" 2>/dev/null; do
            printf "\r${YELLOW}🔨 Building NeonSearch ($OS release mode)... %s${NC}" "${SPIN_CHARS:i++%4:1}"
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

# Copy binary to dist folder with OS-appropriate naming
SOURCE_BINARY="target/release/neonsearch${BINARY_EXT}"
DEST_BINARY="dist/${BINARY_NAME}${BINARY_EXT}"

if [ -f "$SOURCE_BINARY" ]; then
    cp "$SOURCE_BINARY" "$DEST_BINARY"
    echo -e "${GREEN}📦 Binary copied to $DEST_BINARY${NC}"
    
    # Make executable on Unix systems
    if [[ "$OS" != "windows" ]]; then
        chmod +x "$DEST_BINARY"
    fi
else
    echo -e "${RED}❌ Binary not found at $SOURCE_BINARY${NC}"
    exit 1
fi

echo
echo -e "${GREEN}🎉 Build completed successfully for $OS!${NC}"
echo -e "${BLUE}📍 Run with: ./run.sh${NC}"
echo -e "${BLUE}📍 Or directly: ./$DEST_BINARY${NC}"