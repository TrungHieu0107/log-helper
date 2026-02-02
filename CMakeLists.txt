cmake_minimum_required(VERSION 3.20)
project(sql_log_parser VERSION 2.0.0 LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

# MSVC settings
if(MSVC)
    # Use static runtime for Release, dynamic for Debug
    set(CMAKE_MSVC_RUNTIME_LIBRARY "MultiThreaded$<$<CONFIG:Debug>:Debug>")
    add_compile_options(/utf-8)
    # Ensure proper subsystem
    add_link_options(/SUBSYSTEM:WINDOWS)
endif()

# MinGW static linking
if(MINGW)
    set(CMAKE_EXE_LINKER_FLAGS "${CMAKE_EXE_LINKER_FLAGS} -static -static-libgcc -static-libstdc++")
endif()

# ImGui sources
set(IMGUI_DIR ${CMAKE_SOURCE_DIR}/libs/imgui)
set(IMGUI_SOURCES
    ${IMGUI_DIR}/imgui.cpp
    ${IMGUI_DIR}/imgui_demo.cpp
    ${IMGUI_DIR}/imgui_draw.cpp
    ${IMGUI_DIR}/imgui_tables.cpp
    ${IMGUI_DIR}/imgui_widgets.cpp
    ${IMGUI_DIR}/backends/imgui_impl_win32.cpp
    ${IMGUI_DIR}/backends/imgui_impl_dx11.cpp
)

# Application sources
set(APP_SOURCES
    src/main.cpp
    src/app/Application.cpp
    src/core/LogParser.cpp
    src/core/SqlFormatter.cpp
    src/core/QueryProcessor.cpp
    src/core/HtmlGenerator.cpp
    src/config/ConfigManager.cpp
    src/utils/ClipboardHelper.cpp
    src/utils/FileHelper.cpp
    src/utils/Encoding.cpp
    src/utils/SqlConnector.cpp
    src/ui/MainWindow.cpp
    src/ui/Theme.cpp
)

# Executable
add_executable(sql_log_parser WIN32
    ${APP_SOURCES}
    ${IMGUI_SOURCES}
)

# Include directories
target_include_directories(sql_log_parser PRIVATE
    ${CMAKE_SOURCE_DIR}/src
    ${CMAKE_SOURCE_DIR}/libs
    ${IMGUI_DIR}
    ${IMGUI_DIR}/backends
)

# Link libraries
target_link_libraries(sql_log_parser PRIVATE
    d3d11
    dxgi
    d3dcompiler
    dwmapi
    user32
    gdi32
    shell32
    ole32
    comdlg32
    imm32
    kernel32
    odbc32
)

# Release optimizations
if(CMAKE_BUILD_TYPE STREQUAL "Release")
    if(MSVC)
        target_compile_options(sql_log_parser PRIVATE /O2 /GL)
        target_link_options(sql_log_parser PRIVATE /LTCG)
    else()
        target_compile_options(sql_log_parser PRIVATE -O3 -flto)
        target_link_options(sql_log_parser PRIVATE -flto)
    endif()
endif()

# Output name
set_target_properties(sql_log_parser PROPERTIES
    OUTPUT_NAME "sql_log_parser"
    RUNTIME_OUTPUT_DIRECTORY "${CMAKE_BINARY_DIR}/bin"
)
