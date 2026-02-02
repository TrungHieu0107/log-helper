# SQL Log Parser - PowerShell Build Script
# Run this script to build the C++ application

Write-Host "Building SQL Log Parser (C++)..." -ForegroundColor Cyan
Write-Host ""

# Refresh environment PATH
$env:Path = [System.Environment]::GetEnvironmentVariable("Path","Machine") + ";" + [System.Environment]::GetEnvironmentVariable("Path","User")

# Add CMake to path if not found
$cmake = Get-Command cmake -ErrorAction SilentlyContinue
if (-not $cmake) {
    $cmakePaths = @(
        "C:\Program Files\CMake\bin",
        "$env:LOCALAPPDATA\CMake\bin"
    )
    foreach ($p in $cmakePaths) {
        if (Test-Path "$p\cmake.exe") {
            $env:Path = "$p;$env:Path"
            Write-Host "Found CMake at: $p" -ForegroundColor Green
            break
        }
    }
}

# Find VsDevCmd.bat
$vsDevCmd = Get-ChildItem -Path "C:\Program Files*\Microsoft Visual Studio" -Recurse -Filter "VsDevCmd.bat" -ErrorAction SilentlyContinue | 
            Where-Object { $_.FullName -match "2022" } | 
            Select-Object -First 1 -ExpandProperty FullName

if (-not $vsDevCmd) {
    Write-Host "Visual Studio Build Tools not found!" -ForegroundColor Red
    Write-Host "Please install: winget install Microsoft.VisualStudio.2022.BuildTools"
    exit 1
}

Write-Host "Using: $vsDevCmd" -ForegroundColor Gray

# Set up VS environment and build
$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$buildDir = Join-Path $scriptDir "build"

# Create build directory
if (-not (Test-Path $buildDir)) {
    New-Item -ItemType Directory -Path $buildDir | Out-Null
}

# Run the build in VS Developer environment
$buildScript = @"
@echo off
call "$vsDevCmd" -arch=x64 >nul 2>&1

set PATH=C:\Program Files\CMake\bin;%PATH%

cd /d "$scriptDir"

echo Configuring...
cmake -G "NMake Makefiles" -DCMAKE_BUILD_TYPE=Release -B build -S .
if errorlevel 1 exit /b 1

echo Building...
cmake --build build
if errorlevel 1 exit /b 1

echo Done!
"@

$tempBat = Join-Path $env:TEMP "build_sql_parser.bat"
$buildScript | Out-File -FilePath $tempBat -Encoding ASCII

cmd /c $tempBat

if ($LASTEXITCODE -eq 0) {
    Write-Host ""
    Write-Host "Build successful!" -ForegroundColor Green
    
    $exePaths = @(
        (Join-Path $buildDir "sql_log_parser.exe"),
        (Join-Path $buildDir "bin\Release\sql_log_parser.exe")
    )
    
    foreach ($exe in $exePaths) {
        if (Test-Path $exe) {
            Write-Host "Executable: $exe" -ForegroundColor Cyan
            break
        }
    }
} else {
    Write-Host ""
    Write-Host "Build failed!" -ForegroundColor Red
}

Remove-Item $tempBat -ErrorAction SilentlyContinue
