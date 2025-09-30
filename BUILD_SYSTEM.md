# Cross-Platform Build System

NeonSearch includes a comprehensive cross-platform build system that works seamlessly across macOS, Linux, and Windows. This system automatically detects your operating system and runs the appropriate scripts.

## Quick Start

For first-time setup on any operating system:

```bash
# 1. Install all dependencies (Rust, system libraries, dev tools)
./setup

# 2. Build NeonSearch for your OS
./build

# 3. Run the browser
./run
```

## Universal Commands

These commands work on any operating system and automatically detect your platform:

- **`./setup`** - Installs all required dependencies from scratch
- **`./build`** - Builds NeonSearch with OS-specific optimizations  
- **`./run`** - Launches the browser with appropriate configuration

## Platform-Specific Scripts

If you need more control, you can run platform-specific scripts directly:

### Unix/Linux/macOS (.sh scripts)
- `./dependencies.sh` - Install Rust, system deps, dev tools
- `./build.sh` - Build with Unix/macOS optimizations
- `./run.sh` - Launch browser with Unix/macOS settings

### Windows (.bat scripts)
- `dependencies.bat` - Install dependencies on Windows
- `build.bat` - Build with Windows optimizations
- `run.bat` - Launch browser with Windows settings

## Dependency Management

### What Gets Installed
The dependency scripts install everything needed to build NeonSearch:

#### All Platforms
- **Rust** (latest stable) with rustfmt and clippy
- **Cargo tools** (cargo-watch, cargo-edit, cargo-audit)
- **Build essentials** (compilers, cmake, pkg-config)

#### macOS Specific
- **Homebrew** (if not installed)
- **Graphics libraries** (glfw3 for egui)
- **Command line tools**

#### Linux Specific
- **Graphics/windowing libraries** (X11, Wayland support)
- **System development packages**
- **Distribution-specific package managers** (apt, yum, dnf, pacman)

#### Windows Specific
- **Visual Studio Build Tools** verification
- **Windows SDK** checking
- **Development environment setup**

## Build Process

### Automatic OS Detection
The build system automatically detects:
- macOS (darwin)
- Linux (linux-gnu)
- Windows (msys/cygwin via Git Bash)

### OS-Specific Optimizations
- **macOS**: Sets `OBJC_DISABLE_INITIALIZE_FORK_SAFETY=YES` for egui compatibility
- **Linux**: Configures for both X11 and Wayland support
- **Windows**: Uses appropriate executable extensions (.exe)

### Binary Management
- **Location**: Built binaries are copied to `dist/` folder
- **Naming**: OS-appropriate names (NeonSearch on macOS, neonsearch elsewhere)
- **Permissions**: Automatically set executable permissions on Unix systems

## Error Handling

The build system includes comprehensive error handling:

- **Missing Dependencies** - Guides users to install requirements
- **Build Failures** - Provides helpful error messages and suggestions
- **Binary Not Found** - Automatically triggers rebuild if binary missing
- **OS Detection Issues** - Falls back to manual script selection

## Development Workflow

### Fresh System Setup
```bash
git clone [your-repo]
cd NeonSearch
./setup     # Installs everything needed
./build     # First build (may take 5-10 minutes)
./run       # Launch browser
```

### Daily Development
```bash
./build     # Fast incremental builds
./run       # Quick launch

# Or for development with hot reload
cargo run   # Traditional Rust development mode
```

### Debugging Build Issues
```bash
# Check what's installed
rustc --version
cargo --version

# Clean build
cargo clean
./build

# Verbose build output
cargo build --release --verbose
```

## Advanced Usage

### Manual Dependency Installation
If the automatic installer doesn't work for your system:

#### macOS
```bash
# Install Homebrew
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"

# Install dependencies
brew install cmake pkg-config glfw3

# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

#### Linux (Ubuntu/Debian)
```bash
# Install build tools
sudo apt update
sudo apt install build-essential cmake pkg-config curl git

# Install graphics libraries
sudo apt install libxcb1-dev libxkbcommon-dev libwayland-dev \
    libxrandr-dev libxinerama-dev libxcursor-dev libxi-dev libgl1-mesa-dev

# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

#### Windows
1. Install [Visual Studio Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/)
2. Install [Git for Windows](https://git-scm.com/download/win) (includes Git Bash)
3. Install Rust from [rustup.rs](https://rustup.rs/)

### Custom Build Configuration
You can modify build flags by editing the platform-specific scripts:

- Edit `build.sh` for Unix/macOS customizations
- Edit `build.bat` for Windows customizations
- Build flags are in the `configure_build()` function

## Troubleshooting

### Common Issues

**"Rust not found"**
- Run `./setup` or manually install Rust from rustup.rs
- Restart terminal after installation

**"Build failed"**
- Check error messages for missing system libraries
- Run dependency installer for your OS
- Try `cargo clean` then rebuild

**"Binary not found"**
- Scripts automatically trigger rebuild
- Check if build completed successfully
- Look in `target/release/` and `dist/` folders

**Windows "Visual Studio not found"**
- Install Visual Studio Build Tools or Visual Studio Community
- Ensure C++ build tools are installed
- Restart terminal after installation

### Getting Help

1. Check the error messages - they usually contain helpful guidance
2. Verify all dependencies are installed with `./setup`
3. Try a clean build with `cargo clean && ./build`
4. Check the project issues for known problems
5. Open an issue with your OS, error message, and steps to reproduce

## Contributing

When adding new build features:

1. Test on all three platforms (macOS, Linux, Windows)
2. Update both `.sh` and `.bat` versions
3. Add appropriate error handling
4. Update this documentation
5. Test the universal wrapper scripts

The build system is designed to be maintainable and user-friendly. Keep it simple and add clear error messages for any new requirements.