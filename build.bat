@echo off
setlocal enabledelayedexpansion

REM NeonSearch Cross-Platform Build Script for Windows
REM Builds the NeonSearch browser in release mode

echo.
echo ============================================
echo  Building NeonSearch Browser by NeonDev
echo ============================================
echo.

REM Colors using PowerShell escape sequences
set "PS_RED=[31m"
set "PS_GREEN=[32m"
set "PS_YELLOW=[33m"
set "PS_BLUE=[34m"
set "PS_CYAN=[36m"
set "PS_PURPLE=[35m"
set "PS_NC=[0m"

echo [34mDetected OS: [36mWindows[0m

REM Get project root directory
set "PROJECT_ROOT=%~dp0"
cd /d "%PROJECT_ROOT%"

REM Check if Rust is installed
where cargo >nul 2>&1
if %errorLevel% neq 0 (
    echo [31mRust/Cargo not found. Please install Rust first.[0m
    echo [33mRun dependencies.bat to install all dependencies[0m
    pause
    exit /b 1
)

REM Display Rust version info
echo [34mRust version:[0m
rustc --version
echo [34mCargo version:[0m  
cargo --version
echo.

REM Windows-specific build configuration
echo [36mConfiguring build for Windows...[0m
set "BUILD_FLAGS=--release"
set "BINARY_NAME=neonsearch"
set "BINARY_EXT=.exe"
echo.

REM Build in release mode
echo [33mBuilding NeonSearch (Windows release mode)...[0m
echo.

REM Record start time
set "start_time=%time%"

REM Run the build
cargo build %BUILD_FLAGS%
set "BUILD_STATUS=%errorLevel%"

REM Calculate elapsed time
set "end_time=%time%"

if %BUILD_STATUS% equ 0 (
    echo.
    echo [32mRelease build successful![0m
) else (
    echo.
    echo [31mRelease build failed[0m
    pause
    exit /b %BUILD_STATUS%
)

REM Create dist directory if it doesn't exist
if not exist "dist" mkdir dist

REM Copy binary to dist folder
set "SOURCE_BINARY=target\release\%BINARY_NAME%%BINARY_EXT%"
set "DEST_BINARY=dist\%BINARY_NAME%%BINARY_EXT%"

if exist "%SOURCE_BINARY%" (
    copy "%SOURCE_BINARY%" "%DEST_BINARY%" >nul
    echo [32mBinary copied to %DEST_BINARY%[0m
) else (
    echo [31mBinary not found at %SOURCE_BINARY%[0m
    pause
    exit /b 1
)

echo.
echo [32m============================================[0m
echo [32m   Build completed successfully for Windows![0m  
echo [32m============================================[0m
echo.
echo [34mRun with: [33mrun.bat[0m
echo [34mOr directly: [33m%DEST_BINARY%[0m
echo.

pause
exit /b 0