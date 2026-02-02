@echo off
call "C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\Common7\Tools\VsDevCmd.bat" -arch=x64 >nul 2>&1
set "PATH=C:\Program Files\CMake\bin;%PATH%"
cd /d d:\learn\sql_params\build
nmake
pause
