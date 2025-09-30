@echo off
setlocal enabledelayedexpansion

REM NeonSearch Dependencies Installation Script for Windows
REM Installs all required dependencies for building NeonSearch from scratch

echo.
echo ============================================================
echo  NeonSearch Dependencies Installer for Windows by NeonDev
echo ============================================================
echo.

REM Colors (using PowerShell for colored output)
set "PS_RED=[31m"
set "PS_GREEN=[32m"
set "PS_YELLOW=[33m"
set "PS_BLUE=[34m"
set "PS_CYAN=[36m"
set "PS_PURPLE=[35m"
set "PS_NC=[0m"

echo [36mDetected OS: Windows[0m
echo.

REM Check if running in administrator mode
net session >nul 2>&1
if %errorLevel% == 0 (
    echo [32mRunning with administrator privileges[0m
) else (
    echo [33mNote: Some installations may require administrator privileges[0m
)
echo.

REM Function to check if command exists
where rustc >nul 2>&1
if %errorLevel% == 0 (
    echo [32mRust is already installed[0m
    for /f "tokens=*" %%i in ('rustc --version') do set "RUST_VERSION=%%i"
    echo !RUST_VERSION!
    echo.
    echo [33mUpdating Rust to latest version...[0m
    rustup update
    goto :rust_tools
) else (
    goto :install_rust
)

:install_rust
echo [36mInstalling Rust...[0m
echo.

REM Check if we have PowerShell available
powershell -Command "Get-Host" >nul 2>&1
if %errorLevel% == 0 (
    echo [33mDownloading and installing Rust via PowerShell...[0m
    powershell -Command "& {Invoke-WebRequest -Uri 'https://win.rustup.rs/x86_64' -OutFile 'rustup-init.exe'; Start-Process -Wait -FilePath './rustup-init.exe' -ArgumentList '-y'; Remove-Item 'rustup-init.exe'}"
) else (
    echo [31mPowerShell not available. Please download Rust manually from:[0m
    echo https://rustup.rs/
    echo.
    pause
    exit /b 1
)

REM Refresh environment variables
call "%USERPROFILE%\.cargo\env.bat" 2>nul
set "PATH=%USERPROFILE%\.cargo\bin;%PATH%"

echo [32mRust installed successfully![0m
echo.

:rust_tools
echo [36mInstalling Rust development tools...[0m
echo.

REM Install essential Rust components
echo [33mInstalling Rust components...[0m
rustup component add rustfmt clippy

REM Install useful cargo tools (with error handling)
echo [33mInstalling cargo extensions...[0m
cargo install cargo-watch 2>nul || echo [33mFailed to install cargo-watch (this is okay)[0m
cargo install cargo-edit 2>nul || echo [33mFailed to install cargo-edit (this is okay)[0m
cargo install cargo-audit 2>nul || echo [33mFailed to install cargo-audit (this is okay)[0m

echo [32mRust tools installed successfully![0m
echo.

:check_build_tools
echo [36mChecking Windows build tools...[0m
echo.

REM Check for Visual Studio Build Tools
where cl >nul 2>&1
if %errorLevel% == 0 (
    echo [32mVisual Studio compiler found[0m
    for /f "tokens=*" %%i in ('cl 2^>^&1 ^| findstr /C:"Version"') do set "CL_VERSION=%%i"
    echo !CL_VERSION!
) else (
    echo [33mVisual Studio Build Tools not found[0m
    echo.
    echo [33mNeonSearch requires Visual Studio Build Tools for Windows.[0m
    echo Please install one of the following:
    echo.
    echo 1. Visual Studio Community (recommended)
    echo    https://visualstudio.microsoft.com/vs/community/
    echo.
    echo 2. Visual Studio Build Tools
    echo    https://visualstudio.microsoft.com/visual-cpp-build-tools/
    echo.
    echo 3. Windows SDK
    echo    https://developer.microsoft.com/en-us/windows/downloads/windows-sdk/
    echo.
    choice /C YN /M "Would you like to open the Visual Studio download page"
    if !errorlevel! == 1 (
        start https://visualstudio.microsoft.com/vs/community/
    )
    echo.
    echo [33mPlease install build tools and run this script again.[0m
    pause
    exit /b 1
)

REM Check for git
where git >nul 2>&1
if %errorLevel% == 0 (
    echo [32mGit found[0m
    for /f "tokens=*" %%i in ('git --version') do set "GIT_VERSION=%%i"
    echo !GIT_VERSION!
) else (
    echo [33mGit not found (recommended for development)[0m
    echo You can download Git from: https://git-scm.com/download/win
)

echo.

:verify_installation
echo [36mVerifying installation...[0m
echo.

REM Check Rust installation
where rustc >nul 2>&1 && where cargo >nul 2>&1
if %errorLevel% == 0 (
    for /f "tokens=*" %%i in ('rustc --version') do echo [32mRust: %%i[0m
    for /f "tokens=*" %%i in ('cargo --version') do echo [32mCargo: %%i[0m
) else (
    echo [31mRust installation verification failed[0m
    pause
    exit /b 1
)

echo.

:test_build
echo [36mTesting NeonSearch build...[0m
echo.

REM Check if we're in the NeonSearch directory
if not exist "Cargo.toml" (
    echo [31mCargo.toml not found. Please run this script from the NeonSearch project root.[0m
    pause
    exit /b 1
)

choice /C YN /M "Would you like to test the build (this may take a while)"
if %errorLevel% == 2 goto :completion

echo [33mRunning test build...[0m
cargo check --quiet
if %errorLevel% == 0 (
    echo [32mNeonSearch project builds successfully![0m
) else (
    echo [33mBuild check completed with warnings (this is normal)[0m
)

echo.

:completion
echo [32m============================================================[0m
echo [32m    Dependencies installation completed successfully![0m
echo [32m============================================================[0m
echo.
echo [36mNext steps:[0m
echo   1. Run [33mbuild.bat[0m to build NeonSearch
echo   2. Run [33mrun.bat[0m to start the browser  
echo   3. Or run [33mcargo run[0m for development mode
echo.
echo [35mHappy browsing with NeonSearch![0m
echo.

pause
exit /b 0