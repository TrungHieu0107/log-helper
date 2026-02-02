#include "HtmlGenerator.h"
#include "SqlFormatter.h"
#include "../utils/FileHelper.h"
#include <fstream>
#include <sstream>
#include <ctime>
#include <set>
#include <algorithm>
#include <filesystem>

std::string HtmlGenerator::escapeHtml(const std::string& text) {
    std::string result;
    for (char c : text) {
        switch (c) {
            case '&': result += "&amp;"; break;
            case '<': result += "&lt;"; break;
            case '>': result += "&gt;"; break;
            case '"': result += "&quot;"; break;
            case '\'': result += "&#39;"; break;
            default: result += c;
        }
    }
    return result;
}

std::string HtmlGenerator::escapeJsString(const std::string& text) {
    std::string result;
    for (char c : text) {
        switch (c) {
            case '\'': result += "\\'"; break;
            case '\\': result += "\\\\"; break;
            case '\n': result += "\\n"; break;
            case '\r': result += "\\r"; break;
            default: result += c;
        }
    }
    return result;
}

std::string HtmlGenerator::getCurrentDateTime() {
    std::time_t now = std::time(nullptr);
    std::tm* localTime = std::localtime(&now);
    
    char buffer[64];
    std::strftime(buffer, sizeof(buffer), "%Y/%m/%d %H:%M:%S", localTime);
    return buffer;
}

std::string HtmlGenerator::getShortDaoName(const std::string& daoFile) {
    if (daoFile.empty() || daoFile == "Unknown") return "Unknown";
    
    size_t lastDot = daoFile.rfind('.');
    if (lastDot != std::string::npos) {
        return daoFile.substr(lastDot + 1);
    }
    return daoFile;
}

std::string HtmlGenerator::highlightSql(const std::string& sql) {
    if (sql.empty()) return "";
    
    std::string escaped = escapeHtml(sql);
    
    std::vector<std::string> keywords = {
        "SELECT", "FROM", "WHERE", "AND", "OR", "ORDER BY", "GROUP BY",
        "INSERT", "INTO", "VALUES", "UPDATE", "SET", "DELETE",
        "JOIN", "LEFT", "RIGHT", "INNER", "OUTER", "ON",
        "LIKE", "IN", "NOT", "NULL", "IS", "AS", "DISTINCT",
        "COUNT", "SUM", "AVG", "MAX", "MIN", "HAVING", "LIMIT", "OFFSET"
    };
    
    std::string result = escaped;
    
    std::string temp;
    bool inString = false;
    for (size_t i = 0; i < result.size(); ++i) {
        if (result[i] == '\'' && (i == 0 || result[i-1] != '\\')) {
            if (!inString) {
                temp += "<span class=\"string\">'";
                inString = true;
            } else {
                temp += "'</span>";
                inString = false;
            }
        } else {
            temp += result[i];
        }
    }
    result = temp;
    
    for (const auto& kw : keywords) {
        std::string pattern = kw;
        std::string replacement = "<span class=\"keyword\">" + kw + "</span>";
        
        std::string upperResult = result;
        std::transform(upperResult.begin(), upperResult.end(), 
                       upperResult.begin(), ::toupper);
        
        size_t pos = 0;
        while ((pos = upperResult.find(pattern, pos)) != std::string::npos) {
            bool validStart = (pos == 0 || !std::isalnum(result[pos-1]));
            bool validEnd = (pos + pattern.length() >= result.length() || 
                            !std::isalnum(result[pos + pattern.length()]));
            
            if (validStart && validEnd) {
                result.replace(pos, pattern.length(), replacement);
                upperResult = result;
                std::transform(upperResult.begin(), upperResult.end(), 
                               upperResult.begin(), ::toupper);
                pos += replacement.length();
            } else {
                pos++;
            }
        }
    }
    
    return result;
}

std::string HtmlGenerator::generateNavItem(const Execution& exec, int index) {
    std::stringstream ss;
    std::string shortDao = getShortDaoName(exec.daoFile);
    
    ss << "        <li class=\"nav-item\">";
    ss << "<a href=\"#exec-" << index << "\">";
    ss << "<span class=\"nav-id\">#" << index << " - " << escapeHtml(exec.id) << "</span>";
    ss << "<span class=\"nav-time\">" << escapeHtml(exec.timestamp) << "</span>";
    ss << "<span class=\"nav-dao\" title=\"" << escapeHtml(exec.daoFile) << "\">" 
       << escapeHtml(shortDao) << "</span>";
    ss << "</a></li>\n";
    
    return ss.str();
}

std::string HtmlGenerator::generateExecutionCard(const Execution& exec, int index) {
    std::stringstream ss;
    
    std::string filledQuery = exec.filledSql.empty() ? exec.sql : exec.filledSql;
    std::string escapedQuery = escapeJsString(filledQuery);
    
    ss << "    <div class=\"execution-card\" id=\"exec-" << index << "\">\n";
    ss << "        <div class=\"execution-header\">\n";
    ss << "            <div class=\"execution-meta\">\n";
    ss << "                <div class=\"meta-item\">";
    ss << "<span class=\"icon\">&#128278;</span>";
    ss << "<span class=\"label\">ID:</span>";
    ss << "<span class=\"value\">" << escapeHtml(exec.id) << "</span></div>\n";
    ss << "                <div class=\"meta-item\">";
    ss << "<span class=\"icon\">&#9200;</span>";
    ss << "<span class=\"label\">Timestamp:</span>";
    ss << "<span class=\"value\">" << escapeHtml(exec.timestamp) << "</span></div>\n";
    ss << "                <div class=\"meta-item\">";
    ss << "<span class=\"icon\">&#128193;</span>";
    ss << "<span class=\"label\">DAO:</span>";
    ss << "<span class=\"value\">" << escapeHtml(exec.daoFile) << "</span></div>\n";
    ss << "            </div>\n";
    ss << "            <span class=\"execution-index\">Execution #" << index << "</span>\n";
    ss << "        </div>\n";
    
    ss << "        <div class=\"sql-section\">\n";
    ss << "            <h3>&#128204; SQL Query (Filled)</h3>\n";
    ss << "            <div class=\"sql-code\">\n";
    ss << "                <button class=\"copy-btn\" onclick=\"copyToClipboard(this, '" 
       << escapedQuery << "')\">&#128203; Copy</button>\n";
    ss << highlightSql(SqlFormatter::formatSql(filledQuery)) << "\n";
    ss << "            </div>\n";
    ss << "        </div>\n";
    
    if (!exec.params.empty()) {
        ss << "        <div class=\"params-section\">\n";
        ss << "            <h4>&#128221; Parameters</h4>\n";
        ss << "            <div class=\"params-list\">\n";
        
        for (const auto& param : exec.params) {
            size_t firstColon = param.find(':');
            size_t secondColon = param.find(':', firstColon + 1);
            
            if (firstColon != std::string::npos && secondColon != std::string::npos) {
                std::string pType = param.substr(0, firstColon);
                std::string pIndex = param.substr(firstColon + 1, secondColon - firstColon - 1);
                std::string pValue = param.substr(secondColon + 1);
                
                ss << "                <div class=\"param-item\">";
                ss << "<span class=\"param-index\">[" << pIndex << "]</span>";
                ss << "<span class=\"param-type\">" << escapeHtml(pType) << ":</span>";
                ss << "<span class=\"param-value\">" << escapeHtml(pValue) << "</span>";
                ss << "</div>\n";
            } else {
                ss << "                <div class=\"param-item\">" << escapeHtml(param) << "</div>\n";
            }
        }
        
        ss << "            </div>\n";
        ss << "        </div>\n";
    }
    
    ss << "    </div>\n";
    
    return ss.str();
}

std::string HtmlGenerator::getTemplate() {
    std::stringstream ss;
    
    ss << "<!DOCTYPE html>\n";
    ss << "<html lang=\"en\">\n";
    ss << "<head>\n";
    ss << "    <meta charset=\"UTF-8\">\n";
    ss << "    <meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\">\n";
    ss << "    <title>SQL Log Report - {{TITLE}}</title>\n";
    ss << "    <style>\n";
    ss << "        :root {\n";
    ss << "            --bg-primary: #1e1e2e;\n";
    ss << "            --bg-secondary: #2d2d3f;\n";
    ss << "            --bg-card: #3d3d5c;\n";
    ss << "            --bg-sidebar: #252538;\n";
    ss << "            --text-primary: #e0e0e0;\n";
    ss << "            --text-secondary: #a0a0b0;\n";
    ss << "            --accent-blue: #7aa2f7;\n";
    ss << "            --accent-green: #9ece6a;\n";
    ss << "            --accent-purple: #bb9af7;\n";
    ss << "            --accent-orange: #ff9e64;\n";
    ss << "            --border-color: #4d4d6d;\n";
    ss << "            --sidebar-width: 220px;\n";
    ss << "        }\n";
    ss << "        * { margin: 0; padding: 0; box-sizing: border-box; }\n";
    ss << "        body {\n";
    ss << "            font-family: 'Segoe UI', Tahoma, sans-serif;\n";
    ss << "            background: linear-gradient(135deg, var(--bg-primary) 0%, #1a1a2e 100%);\n";
    ss << "            color: var(--text-primary);\n";
    ss << "            min-height: 100vh;\n";
    ss << "            line-height: 1.4;\n";
    ss << "            font-size: 13px;\n";
    ss << "        }\n";
    ss << "        .sidebar {\n";
    ss << "            position: fixed;\n";
    ss << "            left: 0; top: 0;\n";
    ss << "            width: var(--sidebar-width);\n";
    ss << "            height: 100vh;\n";
    ss << "            background: var(--bg-sidebar);\n";
    ss << "            border-right: 1px solid var(--border-color);\n";
    ss << "            overflow-y: auto;\n";
    ss << "            z-index: 1000;\n";
    ss << "        }\n";
    ss << "        .sidebar-header {\n";
    ss << "            padding: 0.8rem;\n";
    ss << "            background: var(--bg-card);\n";
    ss << "            border-bottom: 1px solid var(--border-color);\n";
    ss << "            position: sticky;\n";
    ss << "            top: 0;\n";
    ss << "        }\n";
    ss << "        .sidebar-header h2 { font-size: 1.1rem; color: var(--accent-blue); margin-bottom: 0.5rem; }\n";
    ss << "        .sidebar-search { padding: 1rem; border-bottom: 1px solid var(--border-color); }\n";
    ss << "        .sidebar-search input {\n";
    ss << "            width: 100%;\n";
    ss << "            padding: 0.5rem 0.75rem;\n";
    ss << "            background: var(--bg-primary);\n";
    ss << "            border: 1px solid var(--border-color);\n";
    ss << "            border-radius: 6px;\n";
    ss << "            color: var(--text-primary);\n";
    ss << "            font-size: 0.85rem;\n";
    ss << "        }\n";
    ss << "        .sidebar-search input:focus { outline: none; border-color: var(--accent-blue); }\n";
    ss << "        .nav-list { list-style: none; padding: 0.5rem 0; }\n";
    ss << "        .nav-item { border-bottom: 1px solid rgba(77, 77, 109, 0.3); }\n";
    ss << "        .nav-item a {\n";
    ss << "            display: block;\n";
    ss << "            padding: 0.75rem 1rem;\n";
    ss << "            color: var(--text-primary);\n";
    ss << "            text-decoration: none;\n";
    ss << "            font-size: 0.85rem;\n";
    ss << "            transition: all 0.2s ease;\n";
    ss << "        }\n";
    ss << "        .nav-item a:hover { background: var(--bg-card); color: var(--accent-blue); }\n";
    ss << "        .nav-item .nav-id { font-weight: bold; color: var(--accent-purple); font-family: monospace; }\n";
    ss << "        .nav-item .nav-dao { display: block; font-size: 0.75rem; color: var(--text-secondary); margin-top: 0.25rem; }\n";
    ss << "        .nav-item .nav-time { font-size: 0.7rem; color: var(--accent-orange); }\n";
    ss << "        .main-content { margin-left: var(--sidebar-width); padding: 1rem; }\n";
    ss << "        header {\n";
    ss << "            text-align: center;\n";
    ss << "            margin-bottom: 1.5rem;\n";
    ss << "            padding: 1rem;\n";
    ss << "            background: var(--bg-secondary);\n";
    ss << "            border-radius: 10px;\n";
    ss << "            box-shadow: 0 4px 16px rgba(0, 0, 0, 0.3);\n";
    ss << "        }\n";
    ss << "        header h1 {\n";
    ss << "            font-size: 1.6rem;\n";
    ss << "            background: linear-gradient(90deg, var(--accent-blue), var(--accent-purple));\n";
    ss << "            -webkit-background-clip: text;\n";
    ss << "            -webkit-text-fill-color: transparent;\n";
    ss << "            background-clip: text;\n";
    ss << "        }\n";
    ss << "        .summary { display: grid; grid-template-columns: repeat(auto-fit, minmax(150px, 1fr)); gap: 0.8rem; margin-bottom: 1.5rem; }\n";
    ss << "        .summary-card { background: var(--bg-secondary); padding: 0.8rem; border-radius: 8px; text-align: center; border: 1px solid var(--border-color); }\n";
    ss << "        .summary-card .value { font-size: 1.5rem; font-weight: bold; color: var(--accent-blue); }\n";
    ss << "        .summary-card .label { color: var(--text-secondary); font-size: 0.75rem; }\n";
    ss << "        .execution-card { background: var(--bg-secondary); border-radius: 10px; margin-bottom: 1rem; overflow: hidden; border: 1px solid var(--border-color); scroll-margin-top: 0.5rem; }\n";
    ss << "        .execution-header { display: flex; justify-content: space-between; padding: 0.6rem 0.8rem; background: var(--bg-card); border-bottom: 1px solid var(--border-color); flex-wrap: wrap; gap: 0.5rem; }\n";
    ss << "        .execution-meta { display: flex; gap: 1rem; flex-wrap: wrap; }\n";
    ss << "        .meta-item { display: flex; align-items: center; gap: 0.3rem; }\n";
    ss << "        .meta-item .label { color: var(--text-secondary); font-size: 0.75rem; }\n";
    ss << "        .meta-item .value { color: var(--accent-green); font-weight: 500; }\n";
    ss << "        .execution-index { background: var(--accent-purple); color: white; padding: 0.3rem 0.6rem; border-radius: 12px; font-weight: bold; font-size: 0.75rem; }\n";
    ss << "        .sql-section { padding: 0.8rem; }\n";
    ss << "        .sql-section h3 { color: var(--accent-blue); margin-bottom: 0.5rem; font-size: 0.9rem; }\n";
    ss << "        .sql-code { background: #1a1a2e; padding: 0.8rem; border-radius: 6px; font-family: 'Consolas', monospace; font-size: 0.8rem; line-height: 1.5; white-space: pre-wrap; word-break: break-all; position: relative; }\n";
    ss << "        .sql-code .keyword { color: var(--accent-purple); font-weight: bold; }\n";
    ss << "        .sql-code .string { color: var(--accent-green); }\n";
    ss << "        .copy-btn { position: absolute; top: 0.3rem; right: 0.3rem; background: var(--accent-blue); color: white; border: none; padding: 0.3rem 0.6rem; border-radius: 4px; cursor: pointer; font-size: 0.75rem; }\n";
    ss << "        .copy-btn:hover { background: var(--accent-purple); }\n";
    ss << "        .copy-btn.copied { background: var(--accent-green); }\n";
    ss << "        .params-section { padding: 0 0.8rem 0.8rem; }\n";
    ss << "        .params-section h4 { color: var(--accent-orange); margin-bottom: 0.5rem; font-size: 0.9rem; }\n";
    ss << "        .params-list { display: grid; grid-template-columns: repeat(auto-fill, minmax(250px, 1fr)); gap: 0.5rem; }\n";
    ss << "        .param-item { background: #1a1a2e; padding: 0.5rem 0.75rem; border-radius: 6px; font-family: 'Consolas', monospace; font-size: 0.8rem; display: flex; gap: 0.5rem; }\n";
    ss << "        .param-index { color: var(--accent-purple); font-weight: bold; }\n";
    ss << "        .param-type { color: var(--text-secondary); }\n";
    ss << "        .param-value { color: var(--accent-green); }\n";
    ss << "        footer { text-align: center; padding: 2rem; color: var(--text-secondary); }\n";
    ss << "        @media (max-width: 1024px) { .sidebar { transform: translateX(-100%); } .main-content { margin-left: 0; } }\n";
    ss << "    </style>\n";
    ss << "</head>\n";
    ss << "<body>\n";
    ss << "    <nav class=\"sidebar\" id=\"sidebar\">\n";
    ss << "        <div class=\"sidebar-header\">\n";
    ss << "            <h2>&#128203; Query Navigation</h2>\n";
    ss << "            <div class=\"count\">{{TOTAL_QUERIES}} queries</div>\n";
    ss << "        </div>\n";
    ss << "        <div class=\"sidebar-search\">\n";
    ss << "            <input type=\"text\" id=\"searchInput\" placeholder=\"Search ID or DAO...\" onkeyup=\"filterNav()\">\n";
    ss << "        </div>\n";
    ss << "        <ul class=\"nav-list\" id=\"navList\">\n";
    ss << "{{NAV_ITEMS}}\n";
    ss << "        </ul>\n";
    ss << "    </nav>\n";
    ss << "    <div class=\"main-content\">\n";
    ss << "        <header>\n";
    ss << "            <h1>&#128269; SQL Log Report</h1>\n";
    ss << "            <p class=\"subtitle\">Generated at {{GENERATED_AT}}</p>\n";
    ss << "        </header>\n";
    ss << "        <div class=\"summary\">\n";
    ss << "            <div class=\"summary-card\">\n";
    ss << "                <div class=\"value\">{{TOTAL_QUERIES}}</div>\n";
    ss << "                <div class=\"label\">Total SQL Executions</div>\n";
    ss << "            </div>\n";
    ss << "            <div class=\"summary-card\">\n";
    ss << "                <div class=\"value\">{{UNIQUE_IDS}}</div>\n";
    ss << "                <div class=\"label\">Unique IDs</div>\n";
    ss << "            </div>\n";
    ss << "            <div class=\"summary-card\">\n";
    ss << "                <div class=\"value\">{{LOG_FILE}}</div>\n";
    ss << "                <div class=\"label\">Source Log File</div>\n";
    ss << "            </div>\n";
    ss << "        </div>\n";
    ss << "{{EXECUTIONS}}\n";
    ss << "        <footer><p>Generated by SQL Log Parser v2.0 (C++)</p></footer>\n";
    ss << "    </div>\n";
    ss << "    <script>\n";
    ss << "        function copyToClipboard(btn, text) {\n";
    ss << "            navigator.clipboard.writeText(text).then(function() {\n";
    ss << "                btn.textContent = 'Copied!';\n";
    ss << "                btn.classList.add('copied');\n";
    ss << "                setTimeout(function() { btn.textContent = 'Copy'; btn.classList.remove('copied'); }, 2000);\n";
    ss << "            });\n";
    ss << "        }\n";
    ss << "        function filterNav() {\n";
    ss << "            var input = document.getElementById('searchInput').value.toLowerCase();\n";
    ss << "            var items = document.querySelectorAll('.nav-item');\n";
    ss << "            for (var i = 0; i < items.length; i++) {\n";
    ss << "                items[i].style.display = items[i].textContent.toLowerCase().indexOf(input) > -1 ? '' : 'none';\n";
    ss << "            }\n";
    ss << "        }\n";
    ss << "    </script>\n";
    ss << "</body>\n";
    ss << "</html>\n";
    
    return ss.str();
}

std::string HtmlGenerator::generateReport(const std::vector<Execution>& executions, 
                                           const HtmlOptions& options) {
    std::string html = getTemplate();
    
    std::stringstream navItems;
    for (size_t i = 0; i < executions.size(); ++i) {
        navItems << generateNavItem(executions[i], static_cast<int>(i + 1));
    }
    
    std::stringstream executionCards;
    for (size_t i = 0; i < executions.size(); ++i) {
        executionCards << generateExecutionCard(executions[i], static_cast<int>(i + 1));
    }
    
    std::set<std::string> uniqueIds;
    for (const auto& exec : executions) {
        uniqueIds.insert(exec.id);
    }
    
    std::string logFileName = options.logFile;
    size_t lastSlash = logFileName.find_last_of("/\\");
    if (lastSlash != std::string::npos) {
        logFileName = logFileName.substr(lastSlash + 1);
    }
    
    auto replaceAll = [](std::string& str, const std::string& from, const std::string& to) {
        size_t pos = 0;
        while ((pos = str.find(from, pos)) != std::string::npos) {
            str.replace(pos, from.length(), to);
            pos += to.length();
        }
    };
    
    replaceAll(html, "{{TITLE}}", options.title);
    replaceAll(html, "{{GENERATED_AT}}", getCurrentDateTime());
    replaceAll(html, "{{TOTAL_QUERIES}}", std::to_string(executions.size()));
    replaceAll(html, "{{UNIQUE_IDS}}", std::to_string(uniqueIds.size()));
    replaceAll(html, "{{LOG_FILE}}", logFileName);
    replaceAll(html, "{{NAV_ITEMS}}", navItems.str());
    replaceAll(html, "{{EXECUTIONS}}", executionCards.str());
    
    return html;
}

bool HtmlGenerator::saveReport(const std::string& html, const std::string& outputPath) {
    try {
        if (std::filesystem::exists(outputPath)) {
            std::filesystem::remove(outputPath);
        }
        
        std::ofstream file(outputPath, std::ios::out | std::ios::binary);
        if (!file) return false;
        
        file << html;
        return true;
    } catch (...) {
        return false;
    }
}
