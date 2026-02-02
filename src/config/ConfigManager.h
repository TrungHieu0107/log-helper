#pragma once
#include <string>
#include <vector>
#include <optional>

// Single database connection configuration
struct DbConnection {
    std::string name;          // Connection display name
    std::string server;
    std::string database;
    std::string username;
    std::string password;
    bool useWindowsAuth = true;
};

struct Config {
    std::string logFilePath;
    std::string htmlOutputPath;
    std::string configFile;
    bool autoCopy = true;

    // Multiple SQL Server connections
    std::vector<DbConnection> connections;
    int activeConnectionIndex = -1;  // -1 = no active connection

    // CSV Export
    std::string csvSeparator = ",";
};

class ConfigManager {
public:
    ConfigManager();
    
    Config load();
    bool save(const Config& config);
    std::string getConfigFilePath() const;

private:
    std::string m_configPath;
    std::string getDefaultLogPath() const;
    std::string getExeDirectory() const;
};
