#include "app/Application.h"

#define WIN32_LEAN_AND_MEAN
#include <windows.h>

int WINAPI WinMain(
    _In_ HINSTANCE hInstance,
    _In_opt_ HINSTANCE hPrevInstance,
    _In_ LPSTR lpCmdLine,
    _In_ int nCmdShow)
{
    // Unused parameters
    (void)hInstance;
    (void)hPrevInstance;
    (void)lpCmdLine;
    (void)nCmdShow;
    
    // Initialize COM for shell dialogs
    HRESULT hr = CoInitializeEx(nullptr, COINIT_APARTMENTTHREADED);
    if (FAILED(hr)) {
        MessageBoxA(nullptr, "Failed to initialize COM", "Error", MB_OK | MB_ICONERROR);
        return 1;
    }
    
    Application app;
    
    if (!app.initialize(700, 500, "SQL Log Parser v2.0")) {
        MessageBoxA(nullptr, "Failed to initialize application.\nThis may be due to DirectX11 not being available.", 
                   "Initialization Error", MB_OK | MB_ICONERROR);
        CoUninitialize();
        return 1;
    }
    
    int result = app.run();
    
    app.shutdown();
    
    CoUninitialize();
    
    return result;
}
