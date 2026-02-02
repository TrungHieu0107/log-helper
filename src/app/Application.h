#pragma once
#include "../ui/MainWindow.h"
#include <d3d11.h>

class Application {
public:
    Application();
    ~Application();
    
    // Initialize Win32 window and DirectX
    bool initialize(int width, int height, const char* title);
    
    // Run main message loop
    int run();
    
    // Cleanup
    void shutdown();

private:
    // Win32
    HWND m_hwnd = nullptr;
    WNDCLASSEXW m_wc = {};
    
    // DirectX 11
    ID3D11Device* m_device = nullptr;
    ID3D11DeviceContext* m_deviceContext = nullptr;
    IDXGISwapChain* m_swapChain = nullptr;
    ID3D11RenderTargetView* m_renderTargetView = nullptr;
    
    // UI
    MainWindow m_mainWindow;
    
    // Helpers
    bool createDeviceD3D();
    void cleanupDeviceD3D();
    void createRenderTarget();
    void cleanupRenderTarget();
    
    // Window procedure
    static LRESULT WINAPI WndProc(HWND hWnd, UINT msg, WPARAM wParam, LPARAM lParam);
};
