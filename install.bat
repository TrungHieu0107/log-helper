@echo off
:: SQL Log Parser - Install Dependencies
:: Installs Rust and other required tools

echo ========================================
echo   SQL Log Parser - Install Tools
echo ========================================
echo.

:: Check for admin rights (needed for some installations)
net session >nul 2>&1
if %ERRORLEVEL% NEQ 0 (
    echo NOTE: Running without admin rights.
    echo       Some installations may require admin privileges.
    echo.
)

:: ========================================
:: 1. Check/Install Rust
:: ========================================
echo [1/3] Checking Rust installation...

where cargo >nul 2>&1
if %ERRORLEVEL% EQU 0 (
    echo       Rust is already installed.
    cargo --version
    echo.
) else (
    echo       Rust is not installed. Installing...
    echo.
    
    :: Try winget first
    where winget >nul 2>&1
    if %ERRORLEVEL% EQU 0 (
        echo       Using winget to install Rust...
        winget install Rustlang.Rustup -e --silent
        if %ERRORLEVEL% EQU 0 (
            echo       Rust installed successfully!
            echo.
            echo       IMPORTANT: Please restart your terminal after this script completes.
            echo.
        ) else (
            echo       winget installation failed. Trying alternative method...
            goto :download_rustup
        )
    ) else (
        :download_rustup
        echo       Downloading rustup-init.exe...
        
        :: Download rustup-init.exe using PowerShell
        powershell -Command "& {Invoke-WebRequest -Uri 'https://win.rustup.rs/x86_64' -OutFile '%TEMP%\rustup-init.exe'}"
        
        if exist "%TEMP%\rustup-init.exe" (
            echo       Running rustup installer...
            "%TEMP%\rustup-init.exe" -y
            if %ERRORLEVEL% EQU 0 (
                echo       Rust installed successfully!
                echo.
                echo       IMPORTANT: Please restart your terminal after this script completes.
                echo.
            ) else (
                echo       ERROR: Rust installation failed.
                echo       Please install manually from: https://rustup.rs/
            )
            del "%TEMP%\rustup-init.exe" >nul 2>&1
        ) else (
            echo       ERROR: Failed to download rustup-init.exe
            echo       Please install Rust manually from: https://rustup.rs/
        )
    )
)

:: ========================================
:: 2. Check Visual Studio Build Tools
:: ========================================
echo [2/3] Checking Visual Studio Build Tools...

:: Check for cl.exe (MSVC compiler)
where cl >nul 2>&1
if %ERRORLEVEL% EQU 0 (
    echo       Visual Studio Build Tools found.
    echo.
) else (
    :: Check common VS installation paths
    if exist "C:\Program Files\Microsoft Visual Studio\2022\BuildTools\VC\Tools\MSVC" (
        echo       Visual Studio Build Tools 2022 found.
        echo.
    ) else if exist "C:\Program Files (x86)\Microsoft Visual Studio\2019\BuildTools\VC\Tools\MSVC" (
        echo       Visual Studio Build Tools 2019 found.
        echo.
    ) else (
        echo       Visual Studio Build Tools not found.
        echo.
        echo       Rust on Windows requires Visual Studio Build Tools.
        echo       The Rust installer should have prompted you to install them.
        echo.
        echo       If not installed, download from:
        echo       https://visualstudio.microsoft.com/visual-cpp-build-tools/
        echo.
        echo       Select "Desktop development with C++" workload.
        echo.
    )
)

:: ========================================
:: 3. Check ODBC Driver (optional, for SQL feature)
:: ========================================
echo [3/3] Checking ODBC Driver for SQL Server (optional)...

reg query "HKLM\SOFTWARE\ODBC\ODBCINST.INI\ODBC Driver 17 for SQL Server" >nul 2>&1
if %ERRORLEVEL% EQU 0 (
    echo       ODBC Driver 17 for SQL Server found.
    echo.
) else (
    reg query "HKLM\SOFTWARE\ODBC\ODBCINST.INI\ODBC Driver 18 for SQL Server" >nul 2>&1
    if %ERRORLEVEL% EQU 0 (
        echo       ODBC Driver 18 for SQL Server found.
        echo.
    ) else (
        echo       ODBC Driver for SQL Server not found.
        echo.
        echo       This is OPTIONAL - only needed for SQL execution feature.
        echo       Download from:
        echo       https://docs.microsoft.com/en-us/sql/connect/odbc/download-odbc-driver-for-sql-server
        echo.
    )
)

:: ========================================
:: Summary
:: ========================================
echo ========================================
echo   Installation Complete!
echo ========================================
echo.
echo Next steps:
echo   1. RESTART your terminal (required for PATH updates)
echo   2. Run: build.bat   (to compile the application)
echo   3. Run: run.bat     (to run the application)
echo.
pause
