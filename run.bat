@echo off
:: SQL Log Parser - Run Script
:: Builds (if needed) and runs the application

echo ========================================
echo   SQL Log Parser - Run
echo ========================================
echo.

:: Check if cargo is available
where cargo >nul 2>&1
if %ERRORLEVEL% NEQ 0 (
    echo ERROR: Rust is not installed or not in PATH.
    echo.
    echo Please install Rust from: https://rustup.rs/
    echo Or run: winget install Rustlang.Rustup
    echo.
    echo After installation, restart your terminal and try again.
    pause
    exit /b 1
)

echo Starting SQL Log Parser...
echo.

:: Run in release mode
cargo run --release

if %ERRORLEVEL% NEQ 0 (
    echo.
    echo Application exited with error.
    pause
)
