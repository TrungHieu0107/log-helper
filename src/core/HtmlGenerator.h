#pragma once
#include "LogParser.h"
#include <string>
#include <vector>

struct HtmlOptions {
    std::string title = "SQL Report";
    std::string logFile;
};

class HtmlGenerator {
public:
    // Generate HTML report from executions
    std::string generateReport(const std::vector<Execution>& executions, 
                                const HtmlOptions& options = {});
    
    // Save HTML to file
    bool saveReport(const std::string& html, const std::string& outputPath);

private:
    std::string getTemplate();
    std::string highlightSql(const std::string& sql);
    std::string generateNavItem(const Execution& exec, int index);
    std::string generateExecutionCard(const Execution& exec, int index);
    std::string escapeHtml(const std::string& text);
    std::string escapeJsString(const std::string& text);
    std::string getCurrentDateTime();
    std::string getShortDaoName(const std::string& daoFile);
};
