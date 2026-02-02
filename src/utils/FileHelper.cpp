#include "FileHelper.h"

namespace fs = std::filesystem;

namespace FileHelper {

bool fileExists(const std::string& path) {
    try {
        return fs::exists(path) && fs::is_regular_file(path);
    } catch (...) {
        return false;
    }
}

bool directoryExists(const std::string& path) {
    try {
        return fs::exists(path) && fs::is_directory(path);
    } catch (...) {
        return false;
    }
}

bool createDirectory(const std::string& path) {
    try {
        return fs::create_directories(path);
    } catch (...) {
        return false;
    }
}

std::string getFileName(const std::string& path) {
    return fs::path(path).filename().string();
}

std::string getDirectory(const std::string& path) {
    return fs::path(path).parent_path().string();
}

} // namespace FileHelper
