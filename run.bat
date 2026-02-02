@echo off
cd /d "%~dp0"

set EXE_RELEASE=build\bin\Release\sql_log_parser.exe
set EXE_DEBUG=build\bin\Debug\sql_log_parser.exe
set EXE_MINGW=build\bin\sql_log_parser.exe

:: Check if already built
if exist "%EXE_RELEASE%" (
    start "" "%EXE_RELEASE%"
    exit /b
)

if exist "%EXE_DEBUG%" (
    start "" "%EXE_DEBUG%"
    exit /b
)

if exist "%EXE_MINGW%" (
    start "" "%EXE_MINGW%"
    exit /b
)

:: Not built yet, need to build first
echo Application not built yet. Building...
echo.

if not exist build mkdir build
cd build

:: Try Visual Studio 2022
cmake .. -G "Visual Studio 17 2022" -A x64 2>nul
if %errorlevel%==0 (
    cmake --build . --config Release
    if exist "bin\Release\sql_log_parser.exe" (
        start "" "bin\Release\sql_log_parser.exe"
        exit /b
    )
)

:: Try Visual Studio 2019
cmake .. -G "Visual Studio 16 2019" -A x64 2>nul
if %errorlevel%==0 (
    cmake --build . --config Release
    if exist "bin\Release\sql_log_parser.exe" (
        start "" "bin\Release\sql_log_parser.exe"
        exit /b
    )
)

:: Try MinGW
cmake .. -G "MinGW Makefiles" -DCMAKE_BUILD_TYPE=Release 2>nul
if %errorlevel%==0 (
    cmake --build .
    if exist "bin\sql_log_parser.exe" (
        start "" "bin\sql_log_parser.exe"
        exit /b
    )
)

echo.
echo Build failed! Please install one of:
echo   - Visual Studio 2019/2022 with C++ workload
echo   - MinGW-w64 from https://winlibs.com/
echo And make sure CMake is installed: https://cmake.org/download/
pause
