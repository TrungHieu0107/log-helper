#include "LogParser.h"
#include "SqlFormatter.h"
#include "../utils/Encoding.h"
#include "../utils/FileHelper.h"
#include <regex>
#include <sstream>
#include <set>
#include <algorithm>

std::vector<std::string> LogParser::splitLines(const std::string& content) {
    std::vector<std::string> lines;
    std::istringstream stream(content);
    std::string line;
    
    while (std::getline(stream, line)) {
        // Remove \r if present (Windows line endings)
        if (!line.empty() && line.back() == '\r') {
            line.pop_back();
        }
        lines.push_back(line);
    }
    
    return lines;
}

std::vector<std::string> LogParser::parseParamsString(const std::string& paramsStr) {
    std::vector<std::string> params;
    std::regex paramRegex(R"(\[([^\]]+)\])");
    
    auto begin = std::sregex_iterator(paramsStr.begin(), paramsStr.end(), paramRegex);
    auto end = std::sregex_iterator();
    
    for (auto it = begin; it != end; ++it) {
        params.push_back((*it)[1].str());
    }
    
    return params;
}

std::string LogParser::findDaoClassName(const std::vector<std::string>& lines, 
                                         size_t sqlLineIndex) {
    // Search in 50 lines after SQL line
    size_t searchEnd = std::min(lines.size() - 1, sqlLineIndex + 50);
    
    std::regex daoRegex(R"(Daoの終了jp\.co\.[^\s,]+?([A-Za-z]+Dao)\b)");
    
    for (size_t i = sqlLineIndex + 1; i <= searchEnd; ++i) {
        std::smatch match;
        if (std::regex_search(lines[i], match, daoRegex)) {
            return match[1].str();
        }
    }
    
    return "Unknown";
}

QueryResult LogParser::parseLogFile(const std::string& logFilePath, 
                                     const std::string& targetId) {
    QueryResult result;
    result.id = targetId;
    
    if (!FileHelper::fileExists(logFilePath)) {
        return result;
    }
    
    std::string content = Encoding::readFileAsUtf8(logFilePath);
    if (content.empty()) {
        return result;
    }
    
    // Find SQL statement for ID
    std::string sqlPatternStr = "id=" + targetId + R"(\s+sql=\s*(.+))";
    std::regex sqlPattern(sqlPatternStr);
    std::smatch sqlMatch;
    
    if (std::regex_search(content, sqlMatch, sqlPattern)) {
        result.sql = sqlMatch[1].str();
        // Trim trailing whitespace
        result.sql.erase(result.sql.find_last_not_of(" \t\n\r") + 1);
        result.found = true;
    }
    
    // Find params for ID
    std::string paramsPatternStr = "id=" + targetId + R"(\s+params=(\[[^\n]+))";
    std::regex paramsPattern(paramsPatternStr);
    std::smatch paramsMatch;
    
    if (std::regex_search(content, paramsMatch, paramsPattern)) {
        result.params = parseParamsString(paramsMatch[1].str());
    }
    
    return result;
}

std::vector<Execution> LogParser::parseLogFileAdvanced(const std::string& logFilePath, 
                                                        const std::string& targetId) {
    std::vector<Execution> executions;
    
    if (!FileHelper::fileExists(logFilePath)) {
        return executions;
    }
    
    std::string content = Encoding::readFileAsUtf8(logFilePath);
    if (content.empty()) {
        return executions;
    }
    
    std::vector<std::string> lines = splitLines(content);
    
    std::string sql;
    std::string timestamp;
    std::string daoFile;
    int sqlLineIndex = -1;
    
    struct ParamsSet {
        std::vector<std::string> params;
        std::string timestamp;
    };
    std::vector<ParamsSet> allParamsSets;
    
    // Full line pattern with timestamp
    std::regex fullLinePattern(
        R"(^(\d{4}/\d{2}/\d{2}\s+\d{2}:\d{2}:\d{2}),\w+,([^,]+),.*id=)" + 
        targetId + R"(\s+sql=\s*(.+))");
    
    // Simple SQL pattern
    std::regex simpleSqlPattern("id=" + targetId + R"(\s+sql=\s*(.+))");
    
    // Params pattern
    std::regex paramsPattern("id=" + targetId + R"(\s+params=(\[[^\n]+))");
    
    // Timestamp pattern
    std::regex timestampPattern(R"(^(\d{4}/\d{2}/\d{2}\s+\d{2}:\d{2}:\d{2}))");
    
    for (size_t i = 0; i < lines.size(); ++i) {
        const std::string& line = lines[i];
        
        // Try full pattern first
        std::smatch fullMatch;
        if (std::regex_search(line, fullMatch, fullLinePattern)) {
            timestamp = fullMatch[1].str();
            sql = fullMatch[3].str();
            sqlLineIndex = static_cast<int>(i);
            daoFile = findDaoClassName(lines, i);
            continue;
        }
        
        // Fallback: simple SQL pattern
        if (sql.empty()) {
            std::smatch simpleMatch;
            if (std::regex_search(line, simpleMatch, simpleSqlPattern)) {
                sql = simpleMatch[1].str();
                sqlLineIndex = static_cast<int>(i);
                
                // Try to find timestamp
                std::smatch tsMatch;
                if (std::regex_search(line, tsMatch, timestampPattern)) {
                    timestamp = tsMatch[1].str();
                } else if (i > 0) {
                    if (std::regex_search(lines[i-1], tsMatch, timestampPattern)) {
                        timestamp = tsMatch[1].str();
                    }
                }
                
                daoFile = findDaoClassName(lines, i);
            }
        }
        
        // Find params
        std::smatch paramsMatch;
        if (std::regex_search(line, paramsMatch, paramsPattern)) {
            ParamsSet ps;
            ps.params = parseParamsString(paramsMatch[1].str());
            
            // Try to get timestamp from this line
            std::smatch tsMatch;
            if (std::regex_search(line, tsMatch, timestampPattern)) {
                ps.timestamp = tsMatch[1].str();
            } else {
                ps.timestamp = timestamp;
            }
            
            allParamsSets.push_back(ps);
        }
    }
    
    // Build executions
    if (!sql.empty()) {
        // Trim SQL
        sql.erase(sql.find_last_not_of(" \t\n\r") + 1);
        
        if (!allParamsSets.empty()) {
            // Create execution for each params set
            int index = 1;
            for (const auto& ps : allParamsSets) {
                Execution exec;
                exec.id = targetId;
                exec.timestamp = ps.timestamp.empty() ? timestamp : ps.timestamp;
                exec.daoFile = daoFile;
                exec.sql = sql;
                exec.params = ps.params;
                exec.executionIndex = index++;
                
                // Fill SQL with params
                try {
                    exec.filledSql = SqlFormatter::replacePlaceholders(sql, ps.params);
                } catch (...) {
                    exec.filledSql = sql;
                }
                
                executions.push_back(exec);
            }
        } else {
            // No params - single execution
            Execution exec;
            exec.id = targetId;
            exec.timestamp = timestamp;
            exec.daoFile = daoFile;
            exec.sql = sql;
            exec.filledSql = sql;
            exec.executionIndex = 1;
            executions.push_back(exec);
        }
    }
    
    return executions;
}

std::vector<IdInfo> LogParser::getAllIds(const std::string& logFilePath) {
    std::vector<IdInfo> ids;
    std::set<std::string> seenIds;
    
    if (!FileHelper::fileExists(logFilePath)) {
        return ids;
    }
    
    std::string content = Encoding::readFileAsUtf8(logFilePath);
    if (content.empty()) {
        return ids;
    }
    
    // Find all IDs with SQL
    std::regex sqlPattern(R"(id=([a-f0-9]+)\s+sql=)");
    auto sqlBegin = std::sregex_iterator(content.begin(), content.end(), sqlPattern);
    auto sqlEnd = std::sregex_iterator();
    
    for (auto it = sqlBegin; it != sqlEnd; ++it) {
        std::string id = (*it)[1].str();
        if (seenIds.find(id) == seenIds.end()) {
            seenIds.insert(id);
            IdInfo info;
            info.id = id;
            info.hasSql = true;
            info.paramsCount = 0;
            ids.push_back(info);
        }
    }
    
    // Count params for each ID
    std::regex paramsPattern(R"(id=([a-f0-9]+)\s+params=)");
    auto paramsBegin = std::sregex_iterator(content.begin(), content.end(), paramsPattern);
    auto paramsEnd = std::sregex_iterator();
    
    for (auto it = paramsBegin; it != paramsEnd; ++it) {
        std::string id = (*it)[1].str();
        for (auto& info : ids) {
            if (info.id == id) {
                info.paramsCount++;
                break;
            }
        }
    }
    
    return ids;
}

QueryResult LogParser::getLastQuery(const std::string& logFilePath) {
    QueryResult result;
    
    if (!FileHelper::fileExists(logFilePath)) {
        return result;
    }
    
    std::string content = Encoding::readFileAsUtf8(logFilePath);
    if (content.empty()) {
        return result;
    }
    
    // Find all SQL statements
    std::regex sqlPattern(R"(id=([^\s]+)\s+sql=\s*(.+?)(?=\n|id=|$))");
    
    std::smatch lastMatch;
    std::string::const_iterator searchStart = content.cbegin();
    
    while (std::regex_search(searchStart, content.cend(), lastMatch, sqlPattern)) {
        result.id = lastMatch[1].str();
        result.sql = lastMatch[2].str();
        result.found = true;
        searchStart = lastMatch.suffix().first;
    }
    
    if (result.found) {
        // Trim SQL
        result.sql.erase(result.sql.find_last_not_of(" \t\n\r") + 1);
        
        // Find params for this ID
        std::string paramsPatternStr = "id=" + result.id + R"(\s+params=(\[[^\n]+))";
        std::regex paramsPattern(paramsPatternStr);
        std::smatch paramsMatch;
        
        if (std::regex_search(content, paramsMatch, paramsPattern)) {
            result.params = parseParamsString(paramsMatch[1].str());
        }
    }
    
    return result;
}
