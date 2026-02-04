@echo off
echo Cleaning build artifacts...
cargo clean

echo Cleaning log files...
if exist *.log del *.log
if exist build_err.txt del build_err.txt
if exist check_error.log del check_error.log
if exist check.log del check.log

echo Clean complete!
pause
