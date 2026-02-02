#include "Theme.h"
#include "imgui.h"

#define WIN32_LEAN_AND_MEAN
#include <windows.h>

namespace Theme {

// Get DPI scale factor
static float getDpiScale() {
    HDC hdc = GetDC(nullptr);
    float dpiX = static_cast<float>(GetDeviceCaps(hdc, LOGPIXELSX));
    ReleaseDC(nullptr, hdc);
    return dpiX / 96.0f; // 96 DPI is standard
}

void applyDarkTheme() {
    ImGuiStyle& style = ImGui::GetStyle();
    ImVec4* colors = style.Colors;
    
    // Get DPI scale
    float scale = getDpiScale();
    
    // Color palette matching HTML report
    ImVec4 bg_primary(0.12f, 0.12f, 0.18f, 1.00f);      // #1e1e2e
    ImVec4 bg_secondary(0.18f, 0.18f, 0.25f, 1.00f);    // #2d2d3f
    ImVec4 bg_card(0.24f, 0.24f, 0.36f, 1.00f);         // #3d3d5c
    ImVec4 text_primary(0.88f, 0.88f, 0.88f, 1.00f);    // #e0e0e0
    ImVec4 text_secondary(0.63f, 0.63f, 0.69f, 1.00f);  // #a0a0b0
    ImVec4 accent_blue(0.48f, 0.64f, 0.97f, 1.00f);     // #7aa2f7
    ImVec4 accent_green(0.62f, 0.81f, 0.42f, 1.00f);    // #9ece6a
    ImVec4 accent_purple(0.73f, 0.60f, 0.97f, 1.00f);   // #bb9af7
    ImVec4 accent_orange(1.00f, 0.62f, 0.39f, 1.00f);   // #ff9e64
    ImVec4 border(0.30f, 0.30f, 0.43f, 1.00f);          // #4d4d6d
    
    // Window
    colors[ImGuiCol_WindowBg] = bg_primary;
    colors[ImGuiCol_ChildBg] = bg_secondary;
    colors[ImGuiCol_PopupBg] = bg_secondary;
    colors[ImGuiCol_Border] = border;
    
    // Text
    colors[ImGuiCol_Text] = text_primary;
    colors[ImGuiCol_TextDisabled] = text_secondary;
    
    // Headers
    colors[ImGuiCol_Header] = bg_card;
    colors[ImGuiCol_HeaderHovered] = accent_blue;
    colors[ImGuiCol_HeaderActive] = accent_purple;
    
    // Buttons
    colors[ImGuiCol_Button] = bg_card;
    colors[ImGuiCol_ButtonHovered] = accent_blue;
    colors[ImGuiCol_ButtonActive] = accent_purple;
    
    // Frame (input fields)
    colors[ImGuiCol_FrameBg] = ImVec4(0.10f, 0.10f, 0.15f, 1.00f);
    colors[ImGuiCol_FrameBgHovered] = bg_card;
    colors[ImGuiCol_FrameBgActive] = border;
    
    // Title bar
    colors[ImGuiCol_TitleBg] = bg_secondary;
    colors[ImGuiCol_TitleBgActive] = bg_card;
    colors[ImGuiCol_TitleBgCollapsed] = bg_primary;
    
    // Tabs
    colors[ImGuiCol_Tab] = bg_secondary;
    colors[ImGuiCol_TabHovered] = accent_blue;
    colors[ImGuiCol_TabActive] = accent_purple;
    colors[ImGuiCol_TabUnfocused] = bg_secondary;
    colors[ImGuiCol_TabUnfocusedActive] = bg_card;
    
    // Scrollbar
    colors[ImGuiCol_ScrollbarBg] = bg_primary;
    colors[ImGuiCol_ScrollbarGrab] = border;
    colors[ImGuiCol_ScrollbarGrabHovered] = accent_blue;
    colors[ImGuiCol_ScrollbarGrabActive] = accent_purple;
    
    // Separator
    colors[ImGuiCol_Separator] = border;
    
    // Checkboxes
    colors[ImGuiCol_CheckMark] = accent_green;
    
    // Slider
    colors[ImGuiCol_SliderGrab] = accent_blue;
    colors[ImGuiCol_SliderGrabActive] = accent_purple;
    
    // Resize grip
    colors[ImGuiCol_ResizeGrip] = bg_card;
    colors[ImGuiCol_ResizeGripHovered] = accent_blue;
    colors[ImGuiCol_ResizeGripActive] = accent_purple;
    
    // Style settings - scaled for DPI
    style.WindowRounding = 8.0f;
    style.ChildRounding = 6.0f;
    style.FrameRounding = 4.0f;
    style.GrabRounding = 4.0f;
    style.PopupRounding = 6.0f;
    style.ScrollbarRounding = 4.0f;
    style.TabRounding = 4.0f;
    
    // Responsive padding and spacing - scaled
    style.WindowPadding = ImVec2(12 * scale, 12 * scale);
    style.FramePadding = ImVec2(8 * scale, 4 * scale);
    style.ItemSpacing = ImVec2(8 * scale, 6 * scale);
    style.ItemInnerSpacing = ImVec2(6 * scale, 4 * scale);
    
    // Scrollbar size - scaled
    style.ScrollbarSize = 14.0f * scale;
    style.GrabMinSize = 12.0f * scale;
    
    style.WindowBorderSize = 1.0f;
    style.FrameBorderSize = 0.0f;
    style.PopupBorderSize = 1.0f;
    
    // Apply global scale for all sizes
    style.ScaleAllSizes(scale);
}

} // namespace Theme

