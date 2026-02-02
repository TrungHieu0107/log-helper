# SQL Log Parser - C++ Build Instructions

## Prerequisites

1. **Visual Studio 2022** (Community edition is free)
   - Or Visual Studio Build Tools with C++ workload
   - Download: https://visualstudio.microsoft.com/downloads/

2. **CMake 3.20+**
   - Download: https://cmake.org/download/
   - Or install via: `winget install CMake.CMake`

## Build Steps

### Option 1: Using Visual Studio Developer Command Prompt

1. Open "Developer Command Prompt for VS 2022"
2. Navigate to the project:
   ```cmd
   cd d:\learn\sql_params\cpp
   ```
3. Create build directory and configure:
   ```cmd
   mkdir build
   cd build
   cmake .. -G "Visual Studio 17 2022" -A x64
   ```
4. Build Release:
   ```cmd
   cmake --build . --config Release
   ```
5. Find executable at: `build\bin\Release\sql_log_parser.exe`

### Option 2: Using CMake GUI

1. Open CMake GUI
2. Set source: `d:\learn\sql_params\cpp`
3. Set build: `d:\learn\sql_params\cpp\build`
4. Click Configure → Select Visual Studio 17 2022
5. Click Generate
6. Click Open Project → Build in Visual Studio

### Option 3: Quick Build Script

Run this in Developer Command Prompt:
```cmd
cd d:\learn\sql_params\cpp
mkdir build && cd build
cmake .. -DCMAKE_BUILD_TYPE=Release
cmake --build . --config Release
```

## Output

The compiled executable will be at:
```
d:\learn\sql_params\cpp\build\bin\Release\sql_log_parser.exe
```

## Expected Size

- Debug: ~3-5 MB
- Release: ~1-2 MB

## Troubleshooting

### "cmake is not recognized"
- Install CMake and add to PATH
- Or run from Visual Studio Developer Command Prompt

### "cl is not recognized"  
- Open Visual Studio Developer Command Prompt instead of regular PowerShell

### DirectX errors
- Windows SDK should be installed with Visual Studio C++ workload
