#include "QueryProcessor.h"
#include "SqlFormatter.h"
#include "../utils/ClipboardHelper.h"

ProcessResult QueryProcessor::processQuery(const std::string& targetId, 
                                            const std::string& logFilePath,
                                            bool autoCopy) {
    ProcessResult result;
    
    result.query = m_parser.parseLogFile(logFilePath, targetId);
    
    if (!result.query.found) {
        result.error = "ID not found: " + targetId;
        return result;
    }
    
    result.formattedSql = SqlFormatter::formatSql(result.query.sql);
    result.formattedParams = SqlFormatter::formatParams(result.query.params);
    result.filledSql = getFilledQuery(result.query);
    
    if (autoCopy && !result.filledSql.empty()) {
        result.copiedToClipboard = ClipboardHelper::copyToClipboard(result.filledSql);
    }
    
    return result;
}

ProcessResult QueryProcessor::processLastQuery(const std::string& logFilePath,
                                                bool autoCopy) {
    ProcessResult result;
    
    result.query = m_parser.getLastQuery(logFilePath);
    
    if (!result.query.found) {
        result.error = "No SQL queries found in log file";
        return result;
    }
    
    result.formattedSql = SqlFormatter::formatSql(result.query.sql);
    result.formattedParams = SqlFormatter::formatParams(result.query.params);
    result.filledSql = getFilledQuery(result.query);
    
    if (autoCopy && !result.filledSql.empty()) {
        result.copiedToClipboard = ClipboardHelper::copyToClipboard(result.filledSql);
    }
    
    return result;
}

std::string QueryProcessor::getFilledQuery(const QueryResult& result) {
    if (result.sql.empty()) {
        return "";
    }
    
    if (result.params.empty()) {
        return result.sql;
    }
    
    try {
        return SqlFormatter::replacePlaceholders(result.sql, result.params);
    } catch (...) {
        return result.sql;
    }
}
