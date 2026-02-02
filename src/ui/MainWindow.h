#pragma once
#include "../config/ConfigManager.h"
#include "../core/LogParser.h"
#include "../core/QueryProcessor.h"
#include "../core/HtmlGenerator.h"
#include "../utils/SqlConnector.h"
#include <string>
#include <vector>

class MainWindow {
public:
    MainWindow();

    // Render one frame
    void render();

    // Get current config
    Config& getConfig() { return m_config; }

    // Check if should quit
    bool shouldQuit() const { return m_shouldQuit; }

private:
    // UI State
    char m_searchId[64] = "";
    std::string m_statusMessage;
    bool m_statusIsError = false;
    bool m_shouldQuit = false;

    // Loading state
    bool m_isLoading = false;
    std::string m_loadingMessage;

    // Config
    ConfigManager m_configManager;
    Config m_config;

    // Processors
    LogParser m_parser;
    QueryProcessor m_processor;
    HtmlGenerator m_htmlGenerator;

    // SQL Connection
    SqlConnector m_sqlConnector;
    SqlResult m_queryResult;
    bool m_showConnectionPanel = false;
    int m_editingConnectionIndex = -1;  // -1 = new connection, >= 0 = editing existing
    char m_connName[128] = "";
    char m_sqlServer[256] = "";
    char m_sqlDatabase[128] = "";
    char m_sqlUsername[128] = "";
    char m_sqlPassword[128] = "";
    bool m_sqlUseWindowsAuth = true;
    char m_csvSeparator[8] = ",";

    // Results
    ProcessResult m_lastResult;
    std::vector<IdInfo> m_allIds;
    std::vector<Execution> m_allExecutions;

    // Layout
    float m_leftPanelWidth = 0.55f; // Percentage

    // UI Sections
    void renderHeader();
    void renderToolbar();
    void renderMainContent();
    void renderLeftPanel(float width, float height);
    void renderRightPanel(float width, float height);
    void renderSearchSection();
    void renderResultsSection();
    void renderQueryResult();
    void renderIdsListSection();
    void renderConnectionPanel();
    void renderQueryResultPanel();
    void renderStatusBar();
    void renderLoadingOverlay();

    // Actions
    void searchById();
    void searchLastQuery();
    void loadAllIds();
    void exportHtml(const std::string& targetId = "");
    void exportHtmlAll();
    void copyToClipboard();
    void browseLogFile();
    void browseOutputPath();

    // SQL Actions
    void connectToDatabase();
    void connectToDatabase(int connectionIndex);
    void disconnectFromDatabase();
    void executeCurrentQuery();
    void copyResultAsCsv();

    // Connection management
    void addNewConnection();
    void editConnection(int index);
    void deleteConnection(int index);
    void saveCurrentConnection();

    // Helpers
    void setStatus(const std::string& msg, bool isError = false);
    void setLoading(bool loading, const std::string& message = "");
    void applyThemeOnce();
    void loadConnectionToForm(int index);
    void clearConnectionForm();
};
