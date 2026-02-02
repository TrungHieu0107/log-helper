@echo off
setlocal

echo ========================================
echo SQL Log Parser - C++ Build
echo ========================================
echo.

REM VS Build Tools path
set VSDEVPATH=C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\Common7\Tools\VsDevCmd.bat

if not exist "%VSDEVPATH%" (
    echo ERROR: Visual Studio Build Tools not found!
    pause
    exit /b 1
)

echo Using VS Build Tools...

REM Initialize VS environment
call "%VSDEVPATH%" -arch=x64 -no_logo

REM Add CMake to path
set "PATH=C:\Program Files\CMake\bin;%PATH%"

REM Verify tools
cl >nul 2>&1
if errorlevel 1 (
    echo ERROR: C++ compiler not found!
    pause
    exit /b 1
)

cmake --version >nul 2>&1
if errorlevel 1 (
    echo ERROR: CMake not found!
    pause
    exit /b 1
)

echo Tools ready.
echo.

REM Go to script directory
cd /d "%~dp0"

REM Clean build directory
if exist build (
    echo Cleaning build directory...
    rmdir /s /q build
)
mkdir build
cd build

echo.
echo Configuring with CMake (Visual Studio generator)...
cmake -G "Visual Studio 17 2022" -A x64 ..

if errorlevel 1 (
    echo CMake FAILED!
    cd ..
    pause
    exit /b 1
)

echo.
echo Building Release...
cmake --build . --config Release

if errorlevel 1 (
    echo Build FAILED!
    cd ..
    pause
    exit /b 1
)

cd ..

echo.
echo ========================================
echo BUILD SUCCESSFUL!
echo ========================================
echo.
echo Executable: build\bin\Release\sql_log_parser.exe
echo.

pause
