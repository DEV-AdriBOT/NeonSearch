#!/bin/bash

# NeonSearch Browser Build Script
# Cross-platform build automation for NeonSearch

set -e

echo "🚀 NeonSearch Browser Build Script"
echo "===================================="

# Color codes for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check if Rust is installed
check_rust() {
    if ! command -v cargo &> /dev/null; then
        print_error "Rust/Cargo not found. Please install from https://rustup.rs/"
        exit 1
    fi
    
    local rust_version=$(rustc --version | cut -d' ' -f2)
    print_success "Rust found: $rust_version"
}

# Clean previous builds
clean_build() {
    print_status "Cleaning previous builds..."
    cargo clean
    print_success "Build directory cleaned"
}

# Build for current platform
build_current() {
    print_status "Building NeonSearch for current platform..."
    
    if cargo build --release; then
        print_success "Build completed successfully!"
        
        # Show binary location
        if [[ "$OSTYPE" == "darwin"* ]]; then
            print_status "Binary location: target/release/neonsearch"
        elif [[ "$OSTYPE" == "linux-gnu"* ]]; then
            print_status "Binary location: target/release/neonsearch"
        elif [[ "$OSTYPE" == "msys" ]] || [[ "$OSTYPE" == "win32" ]]; then
            print_status "Binary location: target/release/neonsearch.exe"
        fi
    else
        print_error "Build failed!"
        exit 1
    fi
}

# Build for specific target
build_target() {
    local target=$1
    print_status "Building for target: $target"
    
    # Add target if not already installed
    rustup target add $target 2>/dev/null || true
    
    if cargo build --release --target $target; then
        print_success "Build for $target completed!"
    else
        print_warning "Build for $target failed (this may be due to missing cross-compilation tools)"
    fi
}

# Build for all supported targets
build_all() {
    print_status "Building for all supported platforms..."
    
    # Current common targets
    local targets=(
        "x86_64-apple-darwin"     # macOS Intel
        "aarch64-apple-darwin"    # macOS Apple Silicon
        "x86_64-pc-windows-gnu"   # Windows 64-bit
        "x86_64-unknown-linux-gnu" # Linux 64-bit
    )
    
    for target in "${targets[@]}"; do
        build_target $target
    done
}

# Run tests
run_tests() {
    print_status "Running tests..."
    if cargo test; then
        print_success "All tests passed!"
    else
        print_error "Some tests failed!"
        return 1
    fi
}

# Check code formatting
check_format() {
    print_status "Checking code formatting..."
    if cargo fmt -- --check; then
        print_success "Code formatting is correct!"
    else
        print_warning "Code formatting issues found. Run 'cargo fmt' to fix."
    fi
}

# Run clippy linter
run_clippy() {
    print_status "Running Clippy linter..."
    if cargo clippy -- -D warnings; then
        print_success "No linting issues found!"
    else
        print_warning "Linting issues found. Please review the warnings above."
    fi
}

# Show usage information
show_help() {
    echo "NeonSearch Browser Build Script"
    echo ""
    echo "Usage: $0 [COMMAND]"
    echo ""
    echo "Commands:"
    echo "  build, b       Build for current platform (default)"
    echo "  clean, c       Clean build artifacts"
    echo "  all, a         Build for all supported platforms"
    echo "  test, t        Run tests"
    echo "  format, f      Check code formatting"
    echo "  lint, l        Run clippy linter"
    echo "  check          Run all checks (format, lint, test)"
    echo "  help, h        Show this help message"
    echo ""
    echo "Examples:"
    echo "  $0              # Build for current platform"
    echo "  $0 clean       # Clean build directory"
    echo "  $0 all          # Cross-compile for all targets"
    echo "  $0 check        # Run all checks"
}

# Main script logic
main() {
    # Change to script directory
    cd "$(dirname "$0")"
    
    # Check if we're in a Cargo project
    if [[ ! -f "Cargo.toml" ]]; then
        print_error "Not in a Rust project directory (Cargo.toml not found)"
        exit 1
    fi
    
    # Check Rust installation
    check_rust
    
    # Parse command line arguments
    case "${1:-build}" in
        "build"|"b")
            build_current
            ;;
        "clean"|"c")
            clean_build
            ;;
        "all"|"a")
            build_all
            ;;
        "test"|"t")
            run_tests
            ;;
        "format"|"f")
            check_format
            ;;
        "lint"|"l")
            run_clippy
            ;;
        "check")
            check_format
            run_clippy
            run_tests
            ;;
        "help"|"h"|"--help")
            show_help
            ;;
        *)
            print_error "Unknown command: $1"
            show_help
            exit 1
            ;;
    esac
}

# Run main function with all arguments
main "$@"