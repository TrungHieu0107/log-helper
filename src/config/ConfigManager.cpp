#include "ConfigManager.h"
#include <fstream>
#include <filesystem>
#include <nlohmann_json/json.hpp>

#define WIN32_LEAN_AND_MEAN
#include <windows.h>

namespace fs = std::filesystem;
using json = nlohmann::json;

ConfigManager::ConfigManager() {
    m_configPath = getExeDirectory() + "\\log_parser_config.json";
}

std::string ConfigManager::getExeDirectory() const {
    char path[MAX_PATH];
    GetModuleFileNameA(nullptr, path, MAX_PATH);
    return fs::path(path).parent_path().string();
}

std::string ConfigManager::getDefaultLogPath() const {
    return getExeDirectory() + "\\stcApp.log";
}

std::string ConfigManager::getConfigFilePath() const {
    return m_configPath;
}

Config ConfigManager::load() {
    Config config;
    config.configFile = m_configPath;

    try {
        if (fs::exists(m_configPath)) {
            std::ifstream file(m_configPath);
            json j;
            file >> j;

            if (j.contains("logFilePath")) {
                config.logFilePath = j["logFilePath"].get<std::string>();
            }
            if (j.contains("htmlOutputPath")) {
                config.htmlOutputPath = j["htmlOutputPath"].get<std::string>();
            }
            if (j.contains("autoCopy")) {
                config.autoCopy = j["autoCopy"].get<bool>();
            }
            if (j.contains("csvSeparator")) {
                config.csvSeparator = j["csvSeparator"].get<std::string>();
            }

            // Load multiple connections
            if (j.contains("connections") && j["connections"].is_array()) {
                for (const auto& connJson : j["connections"]) {
                    DbConnection conn;
                    if (connJson.contains("name")) {
                        conn.name = connJson["name"].get<std::string>();
                    }
                    if (connJson.contains("server")) {
                        conn.server = connJson["server"].get<std::string>();
                    }
                    if (connJson.contains("database")) {
                        conn.database = connJson["database"].get<std::string>();
                    }
                    if (connJson.contains("username")) {
                        conn.username = connJson["username"].get<std::string>();
                    }
                    if (connJson.contains("password")) {
                        conn.password = connJson["password"].get<std::string>();
                    }
                    if (connJson.contains("useWindowsAuth")) {
                        conn.useWindowsAuth = connJson["useWindowsAuth"].get<bool>();
                    }
                    config.connections.push_back(conn);
                }
            }

            if (j.contains("activeConnectionIndex")) {
                config.activeConnectionIndex = j["activeConnectionIndex"].get<int>();
            }

            // Migration: convert old single connection to new format
            if (config.connections.empty()) {
                if (j.contains("sqlServer") && !j["sqlServer"].get<std::string>().empty()) {
                    DbConnection conn;
                    conn.name = "Default";
                    conn.server = j["sqlServer"].get<std::string>();
                    if (j.contains("sqlDatabase")) {
                        conn.database = j["sqlDatabase"].get<std::string>();
                    }
                    if (j.contains("sqlUsername")) {
                        conn.username = j["sqlUsername"].get<std::string>();
                    }
                    if (j.contains("sqlPassword")) {
                        conn.password = j["sqlPassword"].get<std::string>();
                    }
                    if (j.contains("sqlUseWindowsAuth")) {
                        conn.useWindowsAuth = j["sqlUseWindowsAuth"].get<bool>();
                    }
                    config.connections.push_back(conn);
                    config.activeConnectionIndex = 0;
                }
            }
        }
    } catch (...) {
        // Use defaults on error
    }

    // Set defaults if empty
    if (config.logFilePath.empty()) {
        config.logFilePath = getDefaultLogPath();
    }
    if (config.htmlOutputPath.empty()) {
        config.htmlOutputPath = getExeDirectory();
    }
    if (config.csvSeparator.empty()) {
        config.csvSeparator = ",";
    }

    return config;
}

bool ConfigManager::save(const Config& config) {
    try {
        json j;
        j["logFilePath"] = config.logFilePath;
        j["htmlOutputPath"] = config.htmlOutputPath;
        j["autoCopy"] = config.autoCopy;
        j["csvSeparator"] = config.csvSeparator;

        // Save multiple connections
        json connectionsArray = json::array();
        for (const auto& conn : config.connections) {
            json connJson;
            connJson["name"] = conn.name;
            connJson["server"] = conn.server;
            connJson["database"] = conn.database;
            connJson["username"] = conn.username;
            connJson["password"] = conn.password;
            connJson["useWindowsAuth"] = conn.useWindowsAuth;
            connectionsArray.push_back(connJson);
        }
        j["connections"] = connectionsArray;
        j["activeConnectionIndex"] = config.activeConnectionIndex;

        std::ofstream file(m_configPath);
        file << j.dump(2);
        return true;
    } catch (...) {
        return false;
    }
}
