#pragma once
#include <string>
#include <vector>
#include <optional>

// Query result for single ID search
struct QueryResult {
    std::string id;
    std::string sql;
    std::vector<std::string> params;
    bool found = false;
};

// Execution info for advanced parsing
struct Execution {
    std::string id;
    std::string timestamp;
    std::string daoFile;
    std::string sql;
    std::string filledSql;
    std::vector<std::string> params;
    int executionIndex = 1;
};

// ID info for listing all IDs
struct IdInfo {
    std::string id;
    bool hasSql = false;
    int paramsCount = 0;
};

class LogParser {
public:
    // Parse log file and get SQL/params for specific ID
    QueryResult parseLogFile(const std::string& logFilePath, 
                              const std::string& targetId);
    
    // Parse with multi-params support (1 SQL with multiple param sets)
    std::vector<Execution> parseLogFileAdvanced(const std::string& logFilePath, 
                                                  const std::string& targetId);
    
    // Get all unique IDs from log file
    std::vector<IdInfo> getAllIds(const std::string& logFilePath);
    
    // Get the last SQL query from log file
    QueryResult getLastQuery(const std::string& logFilePath);

private:
    // Find DAO class name from lines after SQL
    std::string findDaoClassName(const std::vector<std::string>& lines, 
                                  size_t sqlLineIndex);
    
    // Split content into lines
    std::vector<std::string> splitLines(const std::string& content);
    
    // Parse params string like [type:index:value]
    std::vector<std::string> parseParamsString(const std::string& paramsStr);
};
