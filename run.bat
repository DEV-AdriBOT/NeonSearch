@echo off
setlocal enabledelayedexpansion

REM NeonSearch Cross-Platform Run Script for Windows
REM Runs the pre-built NeonSearch browser for faster startup

echo.
echo ==========================================
echo  Starting NeonSearch Browser by NeonDev
echo ==========================================
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

REM Windows-specific binary configuration
set "BINARY_NAME=neonsearch"
set "BINARY_EXT=.exe"

REM Check if release binary exists in multiple locations
set "BINARY_PATH="
set "BINARY_LOCATIONS=dist\%BINARY_NAME%%BINARY_EXT% target\release\neonsearch%BINARY_EXT%"

for %%L in (%BINARY_LOCATIONS%) do (
    if exist "%%L" (
        set "BINARY_PATH=%%L"
        goto :found_binary
    )
)

:not_found
echo [31mNeonSearch binary not found![0m
echo [33mPlease build first with: build.bat[0m
echo.
echo [34mBuilding now...[0m

REM Run the build
call build.bat
if %errorLevel% neq 0 (
    echo [31mBuild failed[0m
    pause
    exit /b 1
)

echo [32mBuild successful![0m

REM Try to find the binary again
for %%L in (%BINARY_LOCATIONS%) do (
    if exist "%%L" (
        set "BINARY_PATH=%%L"
        goto :found_binary
    )
)

echo [31mBinary still not found after build[0m
pause
exit /b 1

:found_binary
echo [32mFound NeonSearch at: %BINARY_PATH%[0m
echo [34mLaunching NeonSearch Browser for Windows...[0m
echo.

REM Run the browser
"%BINARY_PATH%"

REM Check if the browser ran successfully
if %errorLevel% neq 0 (
    echo.
    echo [31mNeonSearch exited with error code %errorLevel%[0m
    pause
)

exit /b 0