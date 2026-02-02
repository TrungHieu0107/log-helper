#pragma once
#include <string>

namespace ClipboardHelper {
    // Copy text to Windows clipboard (UTF-8 input)
    bool copyToClipboard(const std::string& text);
}
