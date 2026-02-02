#include "SqlFormatter.h"
#include <regex>
#include <sstream>
#include <map>
#include <stdexcept>
#include <cctype>

namespace SqlFormatter {

std::string formatSql(const std::string& sql) {
    if (sql.empty()) {
        return "Not found";
    }
    
    std::string formatted = sql;
    std::vector<std::string> keywords = {
        "SELECT", "FROM", "WHERE", "AND", "OR", "ORDER BY", "GROUP BY"
    };
    
    for (const auto& keyword : keywords) {
        std::string pattern = " " + keyword + " ";
        std::string replacement = "\n" + keyword + " ";
        
        size_t pos = 0;
        // Case-insensitive search
        std::string upperFormatted = formatted;
        std::transform(upperFormatted.begin(), upperFormatted.end(), 
                       upperFormatted.begin(), ::toupper);
        
        while ((pos = upperFormatted.find(pattern, pos)) != std::string::npos) {
            formatted.replace(pos, pattern.length(), replacement);
            upperFormatted = formatted;
            std::transform(upperFormatted.begin(), upperFormatted.end(), 
                           upperFormatted.begin(), ::toupper);
            pos += replacement.length();
        }
    }
    
    // Trim leading/trailing whitespace
    size_t start = formatted.find_first_not_of(" \t\n\r");
    size_t end = formatted.find_last_not_of(" \t\n\r");
    
    if (start == std::string::npos) return "";
    return formatted.substr(start, end - start + 1);
}

std::string formatParams(const std::vector<std::string>& params) {
    if (params.empty()) {
        return "Not found";
    }
    
    std::stringstream ss;
    for (const auto& param : params) {
        // Parse format: TYPE:INDEX:VALUE
        size_t firstColon = param.find(':');
        size_t secondColon = param.find(':', firstColon + 1);
        
        if (firstColon != std::string::npos && secondColon != std::string::npos) {
            std::string paramType = param.substr(0, firstColon);
            std::string paramIndex = param.substr(firstColon + 1, 
                                                   secondColon - firstColon - 1);
            std::string paramValue = param.substr(secondColon + 1);
            
            ss << "  [" << paramIndex << "] " << paramType << ": " << paramValue << "\n";
        } else {
            ss << "  " << param << "\n";
        }
    }
    
    return ss.str();
}

std::string replacePlaceholders(const std::string& query, 
                                 const std::vector<std::string>& params) {
    // Build map of position -> value
    std::map<int, std::string> valuesByPos;
    
    for (const auto& param : params) {
        // Parse format: TYPE:INDEX:VALUE
        size_t firstColon = param.find(':');
        size_t secondColon = param.find(':', firstColon + 1);
        
        if (firstColon == std::string::npos || secondColon == std::string::npos) {
            continue;
        }
        
        std::string type = param.substr(0, firstColon);
        int pos = std::stoi(param.substr(firstColon + 1, secondColon - firstColon - 1));
        std::string value = param.substr(secondColon + 1);
        
        // Convert to lowercase for comparison
        std::string typeLower = type;
        std::transform(typeLower.begin(), typeLower.end(), typeLower.begin(), ::tolower);
        
        std::string parsedValue;
        
        if (typeLower == "string") {
            // Escape single quotes for SQL
            std::string escaped;
            for (char c : value) {
                if (c == '\'') escaped += "''";
                else escaped += c;
            }
            parsedValue = "'" + escaped + "'";
        } else if (typeLower == "bigdecimal" || typeLower == "number" || 
                   typeLower == "int" || typeLower == "long" || typeLower == "float") {
            // Numeric values - use as-is
            parsedValue = value;
        } else {
            throw std::runtime_error("Unsupported type: " + type);
        }
        
        valuesByPos[pos] = parsedValue;
    }
    
    // Replace ? placeholders in order
    std::string result;
    int index = 1;
    
    for (size_t i = 0; i < query.size(); ++i) {
        if (query[i] == '?') {
            auto it = valuesByPos.find(index);
            if (it != valuesByPos.end()) {
                result += it->second;
            } else {
                throw std::runtime_error("Missing value for position " + 
                                         std::to_string(index));
            }
            ++index;
        } else {
            result += query[i];
        }
    }
    
    return result;
}

} // namespace SqlFormatter
