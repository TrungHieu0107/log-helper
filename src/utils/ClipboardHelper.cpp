#include "ClipboardHelper.h"
#include "Encoding.h"

#define WIN32_LEAN_AND_MEAN
#include <windows.h>

namespace ClipboardHelper {

bool copyToClipboard(const std::string& text) {
    if (!OpenClipboard(nullptr)) {
        return false;
    }
    
    EmptyClipboard();
    
    // Convert UTF-8 to wide string
    std::wstring wide = Encoding::utf8ToWide(text);
    
    // Allocate global memory for clipboard
    size_t size = (wide.size() + 1) * sizeof(wchar_t);
    HGLOBAL hMem = GlobalAlloc(GMEM_MOVEABLE, size);
    
    if (!hMem) {
        CloseClipboard();
        return false;
    }
    
    // Copy data to global memory
    wchar_t* pMem = static_cast<wchar_t*>(GlobalLock(hMem));
    if (pMem) {
        memcpy(pMem, wide.c_str(), size);
        GlobalUnlock(hMem);
    }
    
    // Set clipboard data
    SetClipboardData(CF_UNICODETEXT, hMem);
    CloseClipboard();
    
    return true;
}

} // namespace ClipboardHelper
