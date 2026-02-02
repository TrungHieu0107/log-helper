@echo off
call "C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\Common7\Tools\VsDevCmd.bat" -arch=x64 >nul 2>&1
set "PATH=C:\Program Files\CMake\bin;%PATH%"
cd /d d:\learn\sql_params

echo Compiling MainWindow.cpp...
echo.

cl /c /EHsc /std:c++17 /I"libs\imgui" /I"libs\imgui\backends" /I"libs\nlohmann_json" src\ui\MainWindow.cpp /Fo:test_compile.obj

echo.
echo Exit code: %ERRORLEVEL%
pause
