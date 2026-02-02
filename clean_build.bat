@echo off
call "C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\Common7\Tools\VsDevCmd.bat" -arch=x64 >nul 2>&1
set "PATH=C:\Program Files\CMake\bin;%PATH%"
cd /d d:\learn\sql_params

echo Cleaning build directory...
rmdir /s /q build 2>nul
mkdir build
cd build

echo.
echo Configuring...
cmake -G "NMake Makefiles" -DCMAKE_BUILD_TYPE=Release .. 2>&1

if errorlevel 1 (
    echo CMake failed!
    pause
    exit /b 1
)

echo.
echo Building...
nmake 2>&1

if errorlevel 1 (
    echo.
    echo Build failed!
    pause
    exit /b 1
)

echo.
echo Build successful!
dir sql_log_parser.exe 2>nul

pause
