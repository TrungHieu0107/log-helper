#pragma once
#include <string>
#include <filesystem>

namespace FileHelper {
    bool fileExists(const std::string& path);
    bool directoryExists(const std::string& path);
    bool createDirectory(const std::string& path);
    std::string getFileName(const std::string& path);
    std::string getDirectory(const std::string& path);
}
