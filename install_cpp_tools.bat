@echo off
echo ========================================
echo Installing Visual C++ Compiler...
echo ========================================
echo.
echo This will download and install the Microsoft C++ compiler.
echo This may take 5-10 minutes depending on your internet speed.
echo.

"C:\Program Files (x86)\Microsoft Visual Studio\Installer\vs_installer.exe" modify ^
    --installPath "C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools" ^
    --add Microsoft.VisualStudio.Component.VC.Tools.x86.x64 ^
    --add Microsoft.VisualStudio.Component.Windows11SDK.22621 ^
    --passive ^
    --wait

echo.
echo Checking if cl.exe is now available...
dir /s "C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\VC\Tools\MSVC\*\bin\Hostx64\x64\cl.exe" 2>nul

if errorlevel 1 (
    echo.
    echo C++ compiler not found. Manual installation may be required.
    echo Please open Visual Studio Installer and add "MSVC v143 - VS 2022 C++ x64/x86 build tools"
) else (
    echo.
    echo C++ compiler installed successfully!
    echo Now run build.bat to compile the application.
)

pause
