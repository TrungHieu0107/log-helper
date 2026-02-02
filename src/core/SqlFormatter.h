#pragma once
#include <string>
#include <vector>

namespace SqlFormatter {
    // Format SQL with line breaks at keywords
    std::string formatSql(const std::string& sql);
    
    // Format params list for display
    std::string formatParams(const std::vector<std::string>& params);
    
    // Replace ? placeholders with actual values from params
    std::string replacePlaceholders(const std::string& query, 
                                     const std::vector<std::string>& params);
}
