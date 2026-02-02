#include "Application.h"
#include "imgui.h"
#include "imgui_impl_win32.h"
#include "imgui_impl_dx11.h"
#include <tchar.h>

// Forward declare message handler from imgui_impl_win32.cpp
extern IMGUI_IMPL_API LRESULT ImGui_ImplWin32_WndProcHandler(HWND hWnd, UINT msg, WPARAM wParam, LPARAM lParam);

// Static pointer for WndProc access
static Application* g_app = nullptr;

Application::Application() {}

Application::~Application() {
    // Don't call shutdown here - it's called explicitly in main
}

bool Application::initialize(int width, int height, const char* title) {
    g_app = this;
    
    // Create application window
    m_wc = { 
        sizeof(m_wc), 
        CS_CLASSDC, 
        WndProc, 
        0L, 
        0L, 
        GetModuleHandle(nullptr), 
        nullptr, 
        nullptr, 
        nullptr, 
        nullptr, 
        L"SqlLogParserClass", 
        nullptr 
    };
    
    ::RegisterClassExW(&m_wc);
    
    // Convert title to wide string
    int wideLen = MultiByteToWideChar(CP_UTF8, 0, title, -1, nullptr, 0);
    std::wstring wideTitle(wideLen, 0);
    MultiByteToWideChar(CP_UTF8, 0, title, -1, wideTitle.data(), wideLen);
    
    // Calculate initial window size (80% of screen)
    int screenWidth = GetSystemMetrics(SM_CXSCREEN);
    int screenHeight = GetSystemMetrics(SM_CYSCREEN);
    int windowWidth = (screenWidth * 80) / 100;
    int windowHeight = (screenHeight * 80) / 100;
    int posX = (screenWidth - windowWidth) / 2;
    int posY = (screenHeight - windowHeight) / 2;
    
    // Use provided size if valid, otherwise use calculated
    if (width > 0 && height > 0) {
        windowWidth = width;
        windowHeight = height;
        posX = (screenWidth - windowWidth) / 2;
        posY = (screenHeight - windowHeight) / 2;
    }
    
    m_hwnd = ::CreateWindowW(
        m_wc.lpszClassName, 
        wideTitle.c_str(),
        WS_OVERLAPPEDWINDOW, 
        posX, posY, 
        windowWidth, windowHeight,
        nullptr, nullptr, 
        m_wc.hInstance, 
        nullptr
    );
    
    // Save initial window style
    m_savedWindowStyle = GetWindowLong(m_hwnd, GWL_STYLE);
    
    // Initialize Direct3D
    if (!createDeviceD3D()) {
        MessageBoxA(nullptr, "Failed to create DirectX11 device.\nPlease ensure your graphics driver is up to date.", 
                   "DirectX Error", MB_OK | MB_ICONERROR);
        cleanupDeviceD3D();
        ::UnregisterClassW(m_wc.lpszClassName, m_wc.hInstance);
        return false;
    }
    
    // Show the window maximized
    ::ShowWindow(m_hwnd, SW_SHOWMAXIMIZED);
    ::UpdateWindow(m_hwnd);
    
    // Setup Dear ImGui context
    IMGUI_CHECKVERSION();
    ImGui::CreateContext();
    ImGuiIO& io = ImGui::GetIO();
    io.ConfigFlags |= ImGuiConfigFlags_NavEnableKeyboard;
    
    // Configure larger font (default is 13px, we add 2px)
    ImFontConfig fontConfig;
    fontConfig.SizePixels = 15.0f;  // 13 + 2 = 15px
    io.Fonts->AddFontDefault(&fontConfig);
    
    // Disable imgui.ini file
    io.IniFilename = nullptr;
    
    // Setup Platform/Renderer backends
    ImGui_ImplWin32_Init(m_hwnd);
    ImGui_ImplDX11_Init(m_device, m_deviceContext);
    
    return true;
}

int Application::run() {
    ImVec4 clear_color = ImVec4(0.12f, 0.12f, 0.18f, 1.00f);
    
    // Main loop
    MSG msg;
    ZeroMemory(&msg, sizeof(msg));
    
    while (msg.message != WM_QUIT) {
        if (::PeekMessage(&msg, nullptr, 0U, 0U, PM_REMOVE)) {
            ::TranslateMessage(&msg);
            ::DispatchMessage(&msg);
            continue;
        }
        
        // Start the Dear ImGui frame
        ImGui_ImplDX11_NewFrame();
        ImGui_ImplWin32_NewFrame();
        ImGui::NewFrame();
        
        // Render our window
        m_mainWindow.render();
        
        // Check if should quit
        if (m_mainWindow.shouldQuit()) {
            ::PostQuitMessage(0);
        }
        
        // Rendering
        ImGui::Render();
        const float clear_color_with_alpha[4] = { 
            clear_color.x * clear_color.w, 
            clear_color.y * clear_color.w, 
            clear_color.z * clear_color.w, 
            clear_color.w 
        };
        m_deviceContext->OMSetRenderTargets(1, &m_renderTargetView, nullptr);
        m_deviceContext->ClearRenderTargetView(m_renderTargetView, clear_color_with_alpha);
        ImGui_ImplDX11_RenderDrawData(ImGui::GetDrawData());
        
        m_swapChain->Present(1, 0); // Present with vsync
    }
    
    return static_cast<int>(msg.wParam);
}

void Application::shutdown() {
    // Only cleanup ImGui if it was initialized
    if (ImGui::GetCurrentContext() != nullptr) {
        ImGui_ImplDX11_Shutdown();
        ImGui_ImplWin32_Shutdown();
        ImGui::DestroyContext();
    }

    cleanupDeviceD3D();

    if (m_hwnd) {
        ::DestroyWindow(m_hwnd);
        m_hwnd = nullptr;
    }

    if (m_wc.lpszClassName) {
        ::UnregisterClassW(m_wc.lpszClassName, m_wc.hInstance);
    }
}

bool Application::createDeviceD3D() {
    DXGI_SWAP_CHAIN_DESC sd;
    ZeroMemory(&sd, sizeof(sd));
    sd.BufferCount = 2;
    sd.BufferDesc.Width = 0;
    sd.BufferDesc.Height = 0;
    sd.BufferDesc.Format = DXGI_FORMAT_R8G8B8A8_UNORM;
    sd.BufferDesc.RefreshRate.Numerator = 60;
    sd.BufferDesc.RefreshRate.Denominator = 1;
    sd.Flags = DXGI_SWAP_CHAIN_FLAG_ALLOW_MODE_SWITCH;
    sd.BufferUsage = DXGI_USAGE_RENDER_TARGET_OUTPUT;
    sd.OutputWindow = m_hwnd;
    sd.SampleDesc.Count = 1;
    sd.SampleDesc.Quality = 0;
    sd.Windowed = TRUE;
    sd.SwapEffect = DXGI_SWAP_EFFECT_DISCARD;
    
    UINT createDeviceFlags = 0;
    D3D_FEATURE_LEVEL featureLevel;
    const D3D_FEATURE_LEVEL featureLevelArray[2] = { 
        D3D_FEATURE_LEVEL_11_0, 
        D3D_FEATURE_LEVEL_10_0 
    };
    
    HRESULT hr = D3D11CreateDeviceAndSwapChain(
        nullptr, 
        D3D_DRIVER_TYPE_HARDWARE, 
        nullptr,
        createDeviceFlags, 
        featureLevelArray, 
        2, 
        D3D11_SDK_VERSION,
        &sd, 
        &m_swapChain, 
        &m_device, 
        &featureLevel, 
        &m_deviceContext
    );
    
    if (FAILED(hr)) {
        return false;
    }
    
    createRenderTarget();
    return true;
}

void Application::cleanupDeviceD3D() {
    cleanupRenderTarget();
    if (m_swapChain) { m_swapChain->Release(); m_swapChain = nullptr; }
    if (m_deviceContext) { m_deviceContext->Release(); m_deviceContext = nullptr; }
    if (m_device) { m_device->Release(); m_device = nullptr; }
}

void Application::createRenderTarget() {
    ID3D11Texture2D* pBackBuffer;
    m_swapChain->GetBuffer(0, IID_PPV_ARGS(&pBackBuffer));
    m_device->CreateRenderTargetView(pBackBuffer, nullptr, &m_renderTargetView);
    pBackBuffer->Release();
}

void Application::cleanupRenderTarget() {
    if (m_renderTargetView) { 
        m_renderTargetView->Release(); 
        m_renderTargetView = nullptr; 
    }
}

void Application::resizeSwapChain(UINT width, UINT height) {
    if (m_swapChain && width > 0 && height > 0) {
        cleanupRenderTarget();
        m_swapChain->ResizeBuffers(0, width, height, DXGI_FORMAT_UNKNOWN, 0);
        createRenderTarget();
    }
}

void Application::toggleFullscreen() {
    if (!m_hwnd) return;
    
    if (!m_isFullscreen) {
        // Save current window placement
        GetWindowPlacement(m_hwnd, &m_savedWindowPlacement);
        m_savedWindowStyle = GetWindowLong(m_hwnd, GWL_STYLE);
        
        // Remove window decorations
        SetWindowLong(m_hwnd, GWL_STYLE, WS_POPUP | WS_VISIBLE);
        
        // Get monitor info for current window
        HMONITOR hMon = MonitorFromWindow(m_hwnd, MONITOR_DEFAULTTONEAREST);
        MONITORINFO mi = { sizeof(mi) };
        GetMonitorInfo(hMon, &mi);
        
        // Set window to cover entire screen
        SetWindowPos(m_hwnd, HWND_TOP,
            mi.rcMonitor.left, mi.rcMonitor.top,
            mi.rcMonitor.right - mi.rcMonitor.left,
            mi.rcMonitor.bottom - mi.rcMonitor.top,
            SWP_FRAMECHANGED | SWP_NOOWNERZORDER);
        
        m_isFullscreen = true;
    } else {
        // Restore window style
        SetWindowLong(m_hwnd, GWL_STYLE, m_savedWindowStyle);
        
        // Restore window placement
        SetWindowPlacement(m_hwnd, &m_savedWindowPlacement);
        
        // Ensure frame is redrawn
        SetWindowPos(m_hwnd, nullptr, 0, 0, 0, 0,
            SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER | SWP_FRAMECHANGED);
        
        m_isFullscreen = false;
    }
}

LRESULT WINAPI Application::WndProc(HWND hWnd, UINT msg, WPARAM wParam, LPARAM lParam) {
    if (ImGui_ImplWin32_WndProcHandler(hWnd, msg, wParam, lParam))
        return true;
    
    switch (msg) {
        case WM_SIZE:
            if (wParam == SIZE_MINIMIZED)
                return 0;
            // Resize swap chain when window is resized
            if (g_app) {
                UINT width = LOWORD(lParam);
                UINT height = HIWORD(lParam);
                g_app->resizeSwapChain(width, height);
            }
            return 0;
            
        case WM_KEYDOWN:
            // F11 toggles fullscreen
            if (wParam == VK_F11 && g_app) {
                g_app->toggleFullscreen();
                return 0;
            }
            // Escape exits fullscreen
            if (wParam == VK_ESCAPE && g_app && g_app->m_isFullscreen) {
                g_app->toggleFullscreen();
                return 0;
            }
            break;
            
        case WM_SYSCOMMAND:
            if ((wParam & 0xfff0) == SC_KEYMENU) // Disable ALT application menu
                return 0;
            break;
            
        case WM_DESTROY:
            g_app = nullptr;
            ::PostQuitMessage(0);
            return 0;
    }
    
    return ::DefWindowProcW(hWnd, msg, wParam, lParam);
}
