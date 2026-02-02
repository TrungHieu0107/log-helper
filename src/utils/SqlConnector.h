#pragma once
#include <string>
#include <vector>
#include <functional>

struct SqlColumn {
    std::string name;
    int type;
    int size;
};

struct SqlResult {
    bool success = false;
    std::string error;
    std::vector<SqlColumn> columns;
    std::vector<std::vector<std::string>> rows;
    int rowsAffected = 0;
};

class SqlConnector {
public:
    SqlConnector();
    ~SqlConnector();

    // Connection
    bool connect(const std::string& server, const std::string& database,
                 const std::string& username, const std::string& password,
                 bool useWindowsAuth);
    void disconnect();
    bool isConnected() const;
    std::string getLastError() const;

    // Query execution
    SqlResult executeQuery(const std::string& sql);

    // Export
    static std::string resultToCsv(const SqlResult& result, const std::string& separator = ",");

private:
    void* m_hEnv = nullptr;
    void* m_hDbc = nullptr;
    bool m_connected = false;
    std::string m_lastError;

    void extractError(void* handle, int handleType);
};
