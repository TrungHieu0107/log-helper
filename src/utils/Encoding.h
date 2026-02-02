#pragma once
#include <string>
#include <vector>

namespace Encoding {
    // Convert SHIFT-JIS (codepage 932) to UTF-8
    std::string shiftJisToUtf8(const std::vector<char>& data);
    std::string shiftJisToUtf8(const std::string& data);
    
    // Read file with SHIFT-JIS encoding and return as UTF-8
    std::string readFileAsUtf8(const std::string& filePath);
    
    // UTF-8 to wide string for Win32 APIs
    std::wstring utf8ToWide(const std::string& str);
    std::string wideToUtf8(const std::wstring& wstr);
}
