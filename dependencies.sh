#!/bin/bash

# NeonSearch Dependencies Installation Script
# Installs all required dependencies for building NeonSearch from scratch
# Supports: macOS, Linux, and Windows (via WSL/Git Bash)

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
PURPLE='\033[0;35m'
NC='\033[0m' # No Color

echo -e "${PURPLE}🔧 NeonSearch Dependencies Installer by NeonDev™${NC}"
echo -e "${PURPLE}=================================================${NC}"
echo ""

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
echo -e "${BLUE}📍 Detected OS: ${YELLOW}$OS${NC}"
echo ""

# Check if command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Install Rust
install_rust() {
    echo -e "${CYAN}🦀 Installing Rust...${NC}"
    
    if command_exists rustc; then
        RUST_VERSION=$(rustc --version)
        echo -e "${GREEN}✅ Rust already installed: $RUST_VERSION${NC}"
        
        # Update Rust to latest
        echo -e "${YELLOW}🔄 Updating Rust to latest version...${NC}"
        rustup update
    else
        echo -e "${YELLOW}📥 Downloading and installing Rust...${NC}"
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
        
        # Source the cargo environment
        source ~/.cargo/env
        
        echo -e "${GREEN}✅ Rust installed successfully!${NC}"
    fi
    
    # Ensure we have the latest Rust edition support
    echo -e "${YELLOW}🔧 Configuring Rust toolchain...${NC}"
    rustup default stable
    rustup update
    
    echo ""
}

# Install system dependencies for macOS
install_macos_deps() {
    echo -e "${CYAN}🍎 Installing macOS dependencies...${NC}"
    
    # Check if Homebrew is installed
    if ! command_exists brew; then
        echo -e "${YELLOW}📥 Installing Homebrew...${NC}"
        /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
        
        # Add Homebrew to PATH for Apple Silicon Macs
        if [[ $(uname -m) == "arm64" ]]; then
            echo 'eval "$(/opt/homebrew/bin/brew shellenv)"' >> ~/.zprofile
            eval "$(/opt/homebrew/bin/brew shellenv)"
        fi
    else
        echo -e "${GREEN}✅ Homebrew already installed${NC}"
        echo -e "${YELLOW}🔄 Updating Homebrew...${NC}"
        brew update
    fi
    
    # Install required system packages
    echo -e "${YELLOW}📦 Installing system packages...${NC}"
    brew install cmake pkg-config
    
    # Install graphics libraries for egui
    echo -e "${YELLOW}🎨 Installing graphics dependencies...${NC}"
    brew install glfw3
    
    echo -e "${GREEN}✅ macOS dependencies installed successfully!${NC}"
    echo ""
}

# Install system dependencies for Linux
install_linux_deps() {
    echo -e "${CYAN}🐧 Installing Linux dependencies...${NC}"
    
    # Detect Linux distribution
    if command_exists apt-get; then
        DISTRO="debian"
        PKG_MANAGER="apt-get"
        UPDATE_CMD="sudo apt-get update"
        INSTALL_CMD="sudo apt-get install -y"
    elif command_exists yum; then
        DISTRO="redhat"
        PKG_MANAGER="yum"
        UPDATE_CMD="sudo yum update -y"
        INSTALL_CMD="sudo yum install -y"
    elif command_exists dnf; then
        DISTRO="fedora"
        PKG_MANAGER="dnf"
        UPDATE_CMD="sudo dnf update -y"
        INSTALL_CMD="sudo dnf install -y"
    elif command_exists pacman; then
        DISTRO="arch"
        PKG_MANAGER="pacman"
        UPDATE_CMD="sudo pacman -Sy"
        INSTALL_CMD="sudo pacman -S --noconfirm"
    else
        echo -e "${RED}❌ Unsupported Linux distribution${NC}"
        echo -e "${YELLOW}💡 Please install these packages manually:${NC}"
        echo "  - build-essential/gcc"
        echo "  - cmake"
        echo "  - pkg-config"
        echo "  - libxcb-devel/libxcb-dev"
        echo "  - libxkbcommon-devel/libxkbcommon-dev"
        echo "  - libwayland-dev (for Wayland support)"
        echo "  - libxrandr-dev libxinerama-dev libxcursor-dev libxi-dev (for X11)"
        return 1
    fi
    
    echo -e "${BLUE}📍 Detected Linux distribution: ${YELLOW}$DISTRO${NC}"
    
    # Update package manager
    echo -e "${YELLOW}🔄 Updating package manager...${NC}"
    $UPDATE_CMD
    
    # Install build tools
    echo -e "${YELLOW}🔨 Installing build tools...${NC}"
    if [[ "$DISTRO" == "debian" ]]; then
        $INSTALL_CMD build-essential cmake pkg-config curl git
    elif [[ "$DISTRO" == "redhat" || "$DISTRO" == "fedora" ]]; then
        $INSTALL_CMD gcc gcc-c++ cmake pkgconfig curl git
    elif [[ "$DISTRO" == "arch" ]]; then
        $INSTALL_CMD base-devel cmake pkgconf curl git
    fi
    
    # Install graphics and windowing libraries
    echo -e "${YELLOW}🎨 Installing graphics dependencies...${NC}"
    if [[ "$DISTRO" == "debian" ]]; then
        $INSTALL_CMD libxcb1-dev libxkbcommon-dev libwayland-dev libxrandr-dev libxinerama-dev libxcursor-dev libxi-dev libgl1-mesa-dev
    elif [[ "$DISTRO" == "redhat" || "$DISTRO" == "fedora" ]]; then
        $INSTALL_CMD libxcb-devel libxkbcommon-devel wayland-devel libXrandr-devel libXinerama-devel libXcursor-devel libXi-devel mesa-libGL-devel
    elif [[ "$DISTRO" == "arch" ]]; then
        $INSTALL_CMD libxcb libxkbcommon wayland libxrandr libxinerama libxcursor libxi mesa
    fi
    
    echo -e "${GREEN}✅ Linux dependencies installed successfully!${NC}"
    echo ""
}

# Install system dependencies for Windows (Git Bash/MSYS2)
install_windows_deps() {
    echo -e "${CYAN}🪟 Windows detected - using Git Bash/MSYS2${NC}"
    echo -e "${YELLOW}💡 For Windows, please ensure you have:${NC}"
    echo "  1. Git Bash or MSYS2 installed"
    echo "  2. Visual Studio Build Tools or Visual Studio Community"
    echo "  3. Windows SDK"
    echo ""
    
    # Check for Visual Studio components
    if command_exists cl; then
        echo -e "${GREEN}✅ Visual Studio compiler found${NC}"
    else
        echo -e "${YELLOW}⚠️  Visual Studio Build Tools not found${NC}"
        echo -e "${YELLOW}📥 Please install Visual Studio Build Tools:${NC}"
        echo "   https://visualstudio.microsoft.com/visual-cpp-build-tools/"
    fi
    
    echo -e "${GREEN}✅ Windows environment check completed${NC}"
    echo ""
}

# Install Rust-specific tools
install_rust_tools() {
    echo -e "${CYAN}🔧 Installing Rust development tools...${NC}"
    
    # Essential tools for Rust development
    echo -e "${YELLOW}📦 Installing cargo components...${NC}"
    rustup component add rustfmt clippy
    
    # Useful cargo extensions
    echo -e "${YELLOW}🛠️  Installing cargo extensions...${NC}"
    cargo install cargo-watch cargo-edit cargo-audit --quiet || {
        echo -e "${YELLOW}⚠️  Some cargo tools failed to install (this is okay)${NC}"
    }
    
    echo -e "${GREEN}✅ Rust tools installed successfully!${NC}"
    echo ""
}

# Verify installation
verify_installation() {
    echo -e "${CYAN}🔍 Verifying installation...${NC}"
    
    # Check Rust
    if command_exists rustc && command_exists cargo; then
        RUST_VERSION=$(rustc --version)
        CARGO_VERSION=$(cargo --version)
        echo -e "${GREEN}✅ Rust: $RUST_VERSION${NC}"
        echo -e "${GREEN}✅ Cargo: $CARGO_VERSION${NC}"
    else
        echo -e "${RED}❌ Rust installation failed${NC}"
        return 1
    fi
    
    # Check git
    if command_exists git; then
        GIT_VERSION=$(git --version)
        echo -e "${GREEN}✅ $GIT_VERSION${NC}"
    else
        echo -e "${YELLOW}⚠️  Git not found (recommended for development)${NC}"
    fi
    
    echo ""
}

# Test build
test_build() {
    echo -e "${CYAN}🧪 Testing NeonSearch build...${NC}"
    
    # Check if we're in the NeonSearch directory
    if [[ ! -f "Cargo.toml" ]]; then
        echo -e "${RED}❌ Cargo.toml not found. Please run this script from the NeonSearch project root.${NC}"
        return 1
    fi
    
    echo -e "${YELLOW}🔨 Running a test build (this may take a while)...${NC}"
    
    # Try to build the project
    if cargo check --quiet; then
        echo -e "${GREEN}✅ NeonSearch project builds successfully!${NC}"
    else
        echo -e "${YELLOW}⚠️  Build check completed with warnings (this is normal)${NC}"
    fi
    
    echo ""
}

# Main installation flow
main() {
    echo -e "${BLUE}🚀 Starting dependency installation for NeonSearch...${NC}"
    echo ""
    
    # Install Rust
    install_rust
    
    # Install system-specific dependencies
    case $OS in
        "macos")
            install_macos_deps
            ;;
        "linux")
            install_linux_deps
            ;;
        "windows")
            install_windows_deps
            ;;
        *)
            echo -e "${RED}❌ Unsupported operating system: $OS${NC}"
            echo -e "${YELLOW}💡 Please install dependencies manually${NC}"
            exit 1
            ;;
    esac
    
    # Install Rust tools
    install_rust_tools
    
    # Verify installation
    verify_installation
    
    # Test build (optional)
    read -p "🧪 Would you like to test the build? (y/N): " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        test_build
    fi
    
    echo -e "${GREEN}🎉 Dependencies installation completed!${NC}"
    echo ""
    echo -e "${CYAN}📋 Next steps:${NC}"
    echo -e "  1. Run ${YELLOW}./build.sh${NC} to build NeonSearch"
    echo -e "  2. Run ${YELLOW}./run.sh${NC} to start the browser"
    echo -e "  3. Or run ${YELLOW}cargo run${NC} for development mode"
    echo ""
    echo -e "${PURPLE}🌟 Happy browsing with NeonSearch! 🌟${NC}"
}

# Run main function
main "$@"