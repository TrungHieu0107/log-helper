#pragma once
#include "LogParser.h"
#include <string>

// Result from processing a query
struct ProcessResult {
    QueryResult query;
    std::string filledSql;
    std::string formattedSql;
    std::string formattedParams;
    bool copiedToClipboard = false;
    std::string error;
};

class QueryProcessor {
public:
    // Process query by ID
    ProcessResult processQuery(const std::string& targetId, 
                                const std::string& logFilePath,
                                bool autoCopy = true);
    
    // Process last query in log file
    ProcessResult processLastQuery(const std::string& logFilePath,
                                    bool autoCopy = true);

private:
    LogParser m_parser;
    
    std::string getFilledQuery(const QueryResult& result);
};
