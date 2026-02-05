@echo off
echo Building Release...
cargo build --release
if %ERRORLEVEL% NEQ 0 (
    echo Build failed!
    pause
    exit /b %ERRORLEVEL%
)
echo Build successful!
echo Output: target\release\sql-log-parser.exe
pause
