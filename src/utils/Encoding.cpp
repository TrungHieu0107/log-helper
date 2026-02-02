#include "Encoding.h"
#include <fstream>

#define WIN32_LEAN_AND_MEAN
#include <windows.h>

namespace Encoding {

std::wstring utf8ToWide(const std::string& str) {
    if (str.empty()) return L"";
    
    int wideLen = MultiByteToWideChar(CP_UTF8, 0, str.c_str(), 
                                       static_cast<int>(str.size()), nullptr, 0);
    if (wideLen == 0) return L"";
    
    std::wstring wide(wideLen, 0);
    MultiByteToWideChar(CP_UTF8, 0, str.c_str(), 
                        static_cast<int>(str.size()), wide.data(), wideLen);
    return wide;
}

std::string wideToUtf8(const std::wstring& wstr) {
    if (wstr.empty()) return "";
    
    int utf8Len = WideCharToMultiByte(CP_UTF8, 0, wstr.c_str(), 
                                       static_cast<int>(wstr.size()), 
                                       nullptr, 0, nullptr, nullptr);
    if (utf8Len == 0) return "";
    
    std::string utf8(utf8Len, 0);
    WideCharToMultiByte(CP_UTF8, 0, wstr.c_str(), 
                        static_cast<int>(wstr.size()), 
                        utf8.data(), utf8Len, nullptr, nullptr);
    return utf8;
}

std::string shiftJisToUtf8(const std::vector<char>& data) {
    if (data.empty()) return "";
    
    // Convert SHIFT-JIS (codepage 932) to UTF-16
    int wideLen = MultiByteToWideChar(932, 0, data.data(), 
                                       static_cast<int>(data.size()), nullptr, 0);
    if (wideLen == 0) return "";
    
    std::wstring wide(wideLen, 0);
    MultiByteToWideChar(932, 0, data.data(), 
                        static_cast<int>(data.size()), wide.data(), wideLen);
    
    // Convert UTF-16 to UTF-8
    return wideToUtf8(wide);
}

std::string shiftJisToUtf8(const std::string& data) {
    return shiftJisToUtf8(std::vector<char>(data.begin(), data.end()));
}

std::string readFileAsUtf8(const std::string& filePath) {
    std::ifstream file(filePath, std::ios::binary);
    if (!file) return "";
    
    // Read entire file into buffer
    std::vector<char> buffer((std::istreambuf_iterator<char>(file)),
                              std::istreambuf_iterator<char>());
    
    // Convert from SHIFT-JIS to UTF-8
    return shiftJisToUtf8(buffer);
}

} // namespace Encoding
