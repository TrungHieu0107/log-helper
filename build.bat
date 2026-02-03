@echo off
:: SQL Log Parser - Build Script
:: Builds the release version of the application

echo ========================================
echo   SQL Log Parser - Build
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

echo [1/2] Building release version...
cargo build --release
if %ERRORLEVEL% NEQ 0 (
    echo.
    echo ERROR: Build failed!
    pause
    exit /b 1
)

echo.
echo [2/2] Build complete!
echo.
echo Output: target\release\sql_log_parser.exe
echo.

:: Show file size
for %%A in (target\release\sql_log_parser.exe) do set SIZE=%%~zA
set /a SIZE_MB=%SIZE%/1024/1024
echo Size: %SIZE_MB% MB

echo.
echo ========================================
echo   Build Successful!
echo ========================================
echo.
pause
