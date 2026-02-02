@echo off
cd /d "%~dp0"

echo ============================================
echo SQL Log Parser - Debug Runner
echo ============================================
echo.

set EXE_RELEASE=build\bin\Release\sql_log_parser.exe
set EXE_DEBUG=build\bin\Debug\sql_log_parser.exe
set EXE_MINGW=build\bin\sql_log_parser.exe

:: Find the exe
set EXE_PATH=

if exist "%EXE_RELEASE%" (
    set EXE_PATH=%EXE_RELEASE%
    echo Found: %EXE_RELEASE%
)

if exist "%EXE_DEBUG%" (
    set EXE_PATH=%EXE_DEBUG%
    echo Found: %EXE_DEBUG%
)

if exist "%EXE_MINGW%" (
    set EXE_PATH=%EXE_MINGW%
    echo Found: %EXE_MINGW%
)

if "%EXE_PATH%"=="" (
    echo ERROR: No executable found! Please build first.
    pause
    exit /b 1
)

echo.
echo Running: %EXE_PATH%
echo.
echo ============================================
echo Application Output:
echo ============================================

:: Run the exe and capture exit code
"%EXE_PATH%"
set EXIT_CODE=%errorlevel%

echo.
echo ============================================
echo Application exited with code: %EXIT_CODE%
echo ============================================

if %EXIT_CODE% NEQ 0 (
    echo.
    echo Possible issues:
    echo   - DirectX 11 not available or driver issue
    echo   - Missing Visual C++ Runtime
    echo   - Antivirus blocking the application
    echo.
    echo Try these fixes:
    echo   1. Update graphics drivers
    echo   2. Install Visual C++ Redistributable 2015-2022:
    echo      https://aka.ms/vs/17/release/vc_redist.x64.exe
    echo   3. Temporarily disable antivirus
    echo   4. Run as Administrator
)

echo.
pause
