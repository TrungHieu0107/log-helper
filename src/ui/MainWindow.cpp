#include "MainWindow.h"
#include "Theme.h"
#include "../utils/ClipboardHelper.h"
#include "../utils/FileHelper.h"
#include "imgui.h"
#include <filesystem>
#include <algorithm>

#define WIN32_LEAN_AND_MEAN
#define NOMINMAX
#include <windows.h>
#include <commdlg.h>
#include <shlobj.h>
#include <shellapi.h>

MainWindow::MainWindow() {
    m_config = m_configManager.load();
    strncpy_s(m_csvSeparator, m_config.csvSeparator.c_str(), sizeof(m_csvSeparator) - 1);
}

void MainWindow::applyThemeOnce() {
    static bool themeApplied = false;
    if (!themeApplied && ImGui::GetCurrentContext() != nullptr) {
        Theme::applyDarkTheme();
        themeApplied = true;
    }
}

void MainWindow::setStatus(const std::string& msg, bool isError) {
    m_statusMessage = msg;
    m_statusIsError = isError;
}

void MainWindow::loadConnectionToForm(int index) {
    if (index >= 0 && index < static_cast<int>(m_config.connections.size())) {
        const auto& conn = m_config.connections[index];
        strncpy_s(m_connName, conn.name.c_str(), sizeof(m_connName) - 1);
        strncpy_s(m_sqlServer, conn.server.c_str(), sizeof(m_sqlServer) - 1);
        strncpy_s(m_sqlDatabase, conn.database.c_str(), sizeof(m_sqlDatabase) - 1);
        strncpy_s(m_sqlUsername, conn.username.c_str(), sizeof(m_sqlUsername) - 1);
        strncpy_s(m_sqlPassword, conn.password.c_str(), sizeof(m_sqlPassword) - 1);
        m_sqlUseWindowsAuth = conn.useWindowsAuth;
    }
}

void MainWindow::clearConnectionForm() {
    m_connName[0] = '\0';
    m_sqlServer[0] = '\0';
    m_sqlDatabase[0] = '\0';
    m_sqlUsername[0] = '\0';
    m_sqlPassword[0] = '\0';
    m_sqlUseWindowsAuth = true;
    m_editingConnectionIndex = -1;
}

void MainWindow::addNewConnection() {
    clearConnectionForm();
    snprintf(m_connName, sizeof(m_connName), "Connection %zu", m_config.connections.size() + 1);
}

void MainWindow::editConnection(int index) {
    m_editingConnectionIndex = index;
    loadConnectionToForm(index);
}

void MainWindow::deleteConnection(int index) {
    if (index >= 0 && index < static_cast<int>(m_config.connections.size())) {
        // Disconnect if this is the active connection
        if (index == m_config.activeConnectionIndex && m_sqlConnector.isConnected()) {
            m_sqlConnector.disconnect();
        }

        m_config.connections.erase(m_config.connections.begin() + index);

        // Adjust active connection index
        if (m_config.activeConnectionIndex == index) {
            m_config.activeConnectionIndex = -1;
        } else if (m_config.activeConnectionIndex > index) {
            m_config.activeConnectionIndex--;
        }

        m_configManager.save(m_config);
        setStatus("Connection deleted");
    }
}

void MainWindow::saveCurrentConnection() {
    DbConnection conn;
    conn.name = m_connName;
    conn.server = m_sqlServer;
    conn.database = m_sqlDatabase;
    conn.username = m_sqlUsername;
    conn.password = m_sqlPassword;
    conn.useWindowsAuth = m_sqlUseWindowsAuth;

    if (conn.name.empty()) {
        conn.name = "Unnamed";
    }

    if (m_editingConnectionIndex >= 0 && m_editingConnectionIndex < static_cast<int>(m_config.connections.size())) {
        // Update existing
        m_config.connections[m_editingConnectionIndex] = conn;
        setStatus("Connection updated: " + conn.name);
    } else {
        // Add new
        m_config.connections.push_back(conn);
        setStatus("Connection added: " + conn.name);
    }

    m_configManager.save(m_config);
    clearConnectionForm();
}

void MainWindow::render() {
    applyThemeOnce();

    const ImGuiViewport* viewport = ImGui::GetMainViewport();
    ImGui::SetNextWindowPos(viewport->WorkPos);
    ImGui::SetNextWindowSize(viewport->WorkSize);

    ImGuiWindowFlags flags = ImGuiWindowFlags_NoTitleBar |
                             ImGuiWindowFlags_NoResize |
                             ImGuiWindowFlags_NoMove |
                             ImGuiWindowFlags_NoCollapse |
                             ImGuiWindowFlags_NoBringToFrontOnFocus;

    ImGui::Begin("SQL Log Parser", nullptr, flags);

    renderHeader();
    ImGui::Separator();
    renderToolbar();
    ImGui::Separator();
    renderSearchSection();
    ImGui::Separator();
    renderMainContent();
    renderStatusBar();

    // Connection panel popup
    if (m_showConnectionPanel) {
        renderConnectionPanel();
    }

    // Loading overlay (rendered last, on top of everything)
    if (m_isLoading) {
        renderLoadingOverlay();
    }

    ImGui::End();
}

void MainWindow::setLoading(bool loading, const std::string& message) {
    m_isLoading = loading;
    m_loadingMessage = message;
}

void MainWindow::renderLoadingOverlay() {
    const ImGuiViewport* viewport = ImGui::GetMainViewport();

    // Semi-transparent overlay
    ImGui::SetNextWindowPos(viewport->WorkPos);
    ImGui::SetNextWindowSize(viewport->WorkSize);
    ImGui::SetNextWindowBgAlpha(0.7f);

    ImGuiWindowFlags overlayFlags = ImGuiWindowFlags_NoTitleBar |
                                     ImGuiWindowFlags_NoResize |
                                     ImGuiWindowFlags_NoMove |
                                     ImGuiWindowFlags_NoScrollbar |
                                     ImGuiWindowFlags_NoInputs;

    ImGui::Begin("##LoadingOverlay", nullptr, overlayFlags);

    float windowWidth = viewport->WorkSize.x;
    float windowHeight = viewport->WorkSize.y;

    // Center the loading content
    float boxWidth = 250.0f;
    float boxHeight = 80.0f;
    float boxX = (windowWidth - boxWidth) * 0.5f;
    float boxY = (windowHeight - boxHeight) * 0.5f;

    ImGui::SetCursorPos(ImVec2(boxX, boxY));

    ImGui::BeginChild("LoadingBox", ImVec2(boxWidth, boxHeight), true, ImGuiWindowFlags_NoScrollbar);

    // Animated spinner using time
    static float rotation = 0.0f;
    rotation += ImGui::GetIO().DeltaTime * 5.0f;
    if (rotation > 6.28318f) rotation -= 6.28318f;

    // Spinner characters
    const char* spinnerChars[] = { "|", "/", "-", "\\" };
    int spinnerIndex = static_cast<int>(rotation * 2.0f) % 4;

    float contentWidth = ImGui::GetContentRegionAvail().x;

    // Spinner
    ImGui::Spacing();
    std::string spinnerText = std::string("  ") + spinnerChars[spinnerIndex] + "  ";
    float spinnerWidth = ImGui::CalcTextSize(spinnerText.c_str()).x;
    ImGui::SetCursorPosX((contentWidth - spinnerWidth) * 0.5f);
    ImGui::TextColored(ImVec4(0.48f, 0.64f, 0.97f, 1.0f), "%s", spinnerText.c_str());

    // Loading message
    ImGui::Spacing();
    std::string displayMessage = m_loadingMessage.empty() ? "Loading..." : m_loadingMessage;
    float textWidth = ImGui::CalcTextSize(displayMessage.c_str()).x;
    ImGui::SetCursorPosX((contentWidth - textWidth) * 0.5f);
    ImGui::Text("%s", displayMessage.c_str());

    ImGui::EndChild();

    ImGui::End();
}

void MainWindow::renderHeader() {
    float windowWidth = ImGui::GetContentRegionAvail().x;
    ImGui::Spacing();

    const char* title = "SQL Log Parser";
    float titleWidth = ImGui::CalcTextSize(title).x;
    ImGui::SetCursorPosX((windowWidth - titleWidth) * 0.5f);
    ImGui::TextColored(ImVec4(0.48f, 0.64f, 0.97f, 1.0f), "%s", title);

    const char* subtitle = "SQL query analyzer and log parser tool";
    float subtitleWidth = ImGui::CalcTextSize(subtitle).x;
    ImGui::SetCursorPosX((windowWidth - subtitleWidth) * 0.5f);
    ImGui::TextDisabled("%s", subtitle);

    ImGui::Spacing();
}

void MainWindow::renderToolbar() {
    ImGui::Spacing();

    float availWidth = ImGui::GetContentRegionAvail().x;
    
    // Calculate dynamic label width based on longest label "Output Dir:"
    float labelWidth = ImGui::CalcTextSize("Output Dir:").x + 20.0f;

    // Calculate button width based on text + padding
    float browseWidth = ImGui::CalcTextSize("Browse").x + ImGui::GetStyle().FramePadding.x * 2 + 16.0f;
    float inputWidth = availWidth - labelWidth - browseWidth - 20.0f;
    if (inputWidth < 200.0f) inputWidth = 200.0f;

    // Log file path
    ImGui::Text("Log File:");
    ImGui::SameLine(labelWidth);
    ImGui::SetNextItemWidth(inputWidth);

    char logPath[512];
    strncpy_s(logPath, m_config.logFilePath.c_str(), sizeof(logPath) - 1);
    if (ImGui::InputText("##logpath", logPath, sizeof(logPath))) {
        m_config.logFilePath = logPath;
        m_configManager.save(m_config);
    }

    ImGui::SameLine();
    if (ImGui::Button("Browse##log", ImVec2(browseWidth, 0))) {
        browseLogFile();
    }

    // Output path
    ImGui::Text("Output Dir:");
    ImGui::SameLine(labelWidth);
    ImGui::SetNextItemWidth(inputWidth);

    char outPath[512];
    strncpy_s(outPath, m_config.htmlOutputPath.c_str(), sizeof(outPath) - 1);
    if (ImGui::InputText("##outpath", outPath, sizeof(outPath))) {
        m_config.htmlOutputPath = outPath;
        m_configManager.save(m_config);
    }

    ImGui::SameLine();
    if (ImGui::Button("Browse##out", ImVec2(browseWidth, 0))) {
        browseOutputPath();
    }

    ImGui::Spacing();
}

void MainWindow::renderSearchSection() {
    ImGui::Spacing();

    const ImGuiStyle& style = ImGui::GetStyle();
    float padding = style.FramePadding.x * 2 + 12.0f;  // Extra padding for comfortable buttons

    // Calculate button widths based on actual text content
    float searchWidth = ImGui::CalcTextSize("Search").x + padding;
    float lastQueryWidth = ImGui::CalcTextSize("Last Query").x + padding;
    float allIdsWidth = ImGui::CalcTextSize("All IDs").x + padding;
    float exportWidth = ImGui::CalcTextSize("Export HTML").x + padding;
    float connectWidth = ImGui::CalcTextSize("DB Connected").x + padding;  // Use longer text for consistent size

    float idInputWidth = 120.0f;

    // Search row
    ImGui::Text("ID:");
    ImGui::SameLine();
    ImGui::SetNextItemWidth(idInputWidth);

    bool enterPressed = ImGui::InputText("##searchid", m_searchId, sizeof(m_searchId),
                                          ImGuiInputTextFlags_EnterReturnsTrue);

    ImGui::SameLine();
    if (ImGui::Button("Search", ImVec2(searchWidth, 0)) || enterPressed) {
        searchById();
    }

    ImGui::SameLine();
    if (ImGui::Button("Last Query", ImVec2(lastQueryWidth, 0))) {
        searchLastQuery();
    }

    ImGui::SameLine();
    if (ImGui::Button("All IDs", ImVec2(allIdsWidth, 0))) {
        loadAllIds();
    }

    ImGui::SameLine();
    if (ImGui::Button("Export HTML", ImVec2(exportWidth, 0))) {
        exportHtmlAll();
    }

    ImGui::SameLine();
    ImGui::Checkbox("Auto-copy", &m_config.autoCopy);

    // Database connection button
    ImGui::SameLine();
    ImGui::TextDisabled("|");
    ImGui::SameLine();

    if (m_sqlConnector.isConnected()) {
        ImGui::PushStyleColor(ImGuiCol_Button, ImVec4(0.2f, 0.5f, 0.2f, 1.0f));
        if (ImGui::Button("DB Connected", ImVec2(connectWidth, 0))) {
            m_showConnectionPanel = true;
        }
        ImGui::PopStyleColor();
    } else {
        if (ImGui::Button("Connect DB", ImVec2(connectWidth, 0))) {
            m_showConnectionPanel = true;
        }
    }

    ImGui::Spacing();
}

void MainWindow::renderMainContent() {
    float availHeight = ImGui::GetContentRegionAvail().y - 30.0f;
    float availWidth = ImGui::GetContentRegionAvail().x;

    if (availHeight < 100.0f) availHeight = 100.0f;

    // Two-panel layout when we have query result
    bool showRightPanel = m_lastResult.query.found || m_queryResult.success;
    
    // Responsive threshold: collapse to single panel on narrow windows
    float minTwoPanelWidth = 650.0f;

    if (showRightPanel && availWidth > minTwoPanelWidth) {
        float leftWidth = availWidth * m_leftPanelWidth - 5.0f;
        float rightWidth = availWidth * (1.0f - m_leftPanelWidth) - 5.0f;

        // Left panel
        renderLeftPanel(leftWidth, availHeight);

        ImGui::SameLine();

        // Splitter
        ImGui::Button("||", ImVec2(8, availHeight));
        if (ImGui::IsItemActive()) {
            float delta = ImGui::GetIO().MouseDelta.x / availWidth;
            m_leftPanelWidth += delta;
            m_leftPanelWidth = std::clamp(m_leftPanelWidth, 0.25f, 0.85f);
        }
        if (ImGui::IsItemHovered()) {
            ImGui::SetMouseCursor(ImGuiMouseCursor_ResizeEW);
        }

        ImGui::SameLine();

        // Right panel
        renderRightPanel(rightWidth, availHeight);
    } else {
        // Single panel layout
        renderLeftPanel(availWidth, availHeight);
    }
}

void MainWindow::renderLeftPanel(float width, float height) {
    ImGui::BeginChild("LeftPanel", ImVec2(width, height), true);

    if (!m_allIds.empty()) {
        renderIdsListSection();
    } else if (m_lastResult.query.found) {
        renderQueryResult();
    } else if (!m_lastResult.error.empty()) {
        ImGui::Spacing();
        ImGui::TextColored(ImVec4(1.0f, 0.4f, 0.4f, 1.0f), "[Error] %s", m_lastResult.error.c_str());
    } else {
        // Welcome message
        ImGui::Spacing();
        float panelWidth = ImGui::GetContentRegionAvail().x;

        const char* welcome = "Welcome to SQL Log Parser!";
        float welcomeWidth = ImGui::CalcTextSize(welcome).x;
        ImGui::SetCursorPosX((panelWidth - welcomeWidth) * 0.5f);
        ImGui::TextColored(ImVec4(0.48f, 0.64f, 0.97f, 1.0f), "%s", welcome);

        ImGui::Spacing();
        ImGui::Spacing();

        ImGui::TextDisabled("How to use:");
        ImGui::Spacing();
        ImGui::BulletText("Enter an ID and click 'Search' to find a query");
        ImGui::BulletText("Click 'Last Query' to view the most recent SQL");
        ImGui::BulletText("Click 'All IDs' to see all available IDs");
        ImGui::BulletText("Click 'Connect DB' to run queries on SQL Server");
    }

    ImGui::EndChild();
}

void MainWindow::renderRightPanel(float width, float height) {
    ImGui::BeginChild("RightPanel", ImVec2(width, height), true);

    ImGui::TextColored(ImVec4(0.48f, 0.64f, 0.97f, 1.0f), "Query Result");
    ImGui::Separator();

    if (!m_sqlConnector.isConnected()) {
        ImGui::Spacing();
        ImGui::TextDisabled("Not connected to database.");
        ImGui::Spacing();
        if (ImGui::Button("Connect to Database")) {
            m_showConnectionPanel = true;
        }
    } else {
        // Execute button
        ImGui::Spacing();
        if (ImGui::Button("Execute Query")) {
            executeCurrentQuery();
        }
        ImGui::SameLine();
        if (ImGui::Button("Copy as CSV")) {
            copyResultAsCsv();
        }
        ImGui::SameLine();
        ImGui::Text("Separator:");
        ImGui::SameLine();
        ImGui::SetNextItemWidth(50.0f);
        if (ImGui::InputText("##sep", m_csvSeparator, sizeof(m_csvSeparator))) {
            m_config.csvSeparator = m_csvSeparator;
            m_configManager.save(m_config);
        }

        ImGui::Spacing();

        // Query result panel
        renderQueryResultPanel();
    }

    ImGui::EndChild();
}

void MainWindow::renderQueryResult() {
    ImGui::TextColored(ImVec4(0.73f, 0.60f, 0.97f, 1.0f), "ID: %s",
                      m_lastResult.query.id.c_str());

    ImGui::Spacing();
    ImGui::Separator();
    ImGui::Spacing();

    if (ImGui::CollapsingHeader("SQL Query (Parameters Filled)", ImGuiTreeNodeFlags_DefaultOpen)) {
        float availHeight = ImGui::GetContentRegionAvail().y;
        float sqlHeight = std::min(200.0f, availHeight * 0.5f);
        if (sqlHeight < 80.0f) sqlHeight = 80.0f;

        ImGui::BeginChild("SqlCode", ImVec2(0, sqlHeight), true,
                          ImGuiWindowFlags_HorizontalScrollbar);

        std::string sql = m_lastResult.filledSql.empty() ?
                          m_lastResult.query.sql : m_lastResult.filledSql;

        ImGui::PushStyleColor(ImGuiCol_Text, ImVec4(0.62f, 0.81f, 0.42f, 1.0f));
        ImGui::TextWrapped("%s", sql.c_str());
        ImGui::PopStyleColor();

        ImGui::EndChild();

        ImGui::Spacing();

        if (ImGui::Button("Copy to Clipboard")) {
            copyToClipboard();
        }
        ImGui::SameLine();
        if (ImGui::Button("Export to HTML")) {
            exportHtml(m_lastResult.query.id);
        }

        if (m_sqlConnector.isConnected()) {
            ImGui::SameLine();
            if (ImGui::Button("Execute on DB")) {
                executeCurrentQuery();
            }
        }
    }

    ImGui::Spacing();

    if (!m_lastResult.query.params.empty()) {
        if (ImGui::CollapsingHeader("Parameters", ImGuiTreeNodeFlags_DefaultOpen)) {
            float paramsHeight = std::min(100.0f, ImGui::GetContentRegionAvail().y - 10.0f);
            if (paramsHeight < 50.0f) paramsHeight = 50.0f;

            ImGui::BeginChild("Params", ImVec2(0, paramsHeight), true);

            ImGui::PushStyleColor(ImGuiCol_Text, ImVec4(1.00f, 0.62f, 0.39f, 1.0f));
            ImGui::TextWrapped("%s", m_lastResult.formattedParams.c_str());
            ImGui::PopStyleColor();

            ImGui::EndChild();
        }
    }
}

void MainWindow::renderIdsListSection() {
    ImGui::TextColored(ImVec4(0.48f, 0.64f, 0.97f, 1.0f),
                       "Found %zu IDs:", m_allIds.size());
    ImGui::Separator();
    ImGui::Spacing();

    ImGuiTableFlags tableFlags = ImGuiTableFlags_Borders |
                                  ImGuiTableFlags_RowBg |
                                  ImGuiTableFlags_ScrollY |
                                  ImGuiTableFlags_Resizable |
                                  ImGuiTableFlags_Reorderable;

    float tableHeight = ImGui::GetContentRegionAvail().y - 35.0f;

    if (ImGui::BeginTable("IdsTable", 3, tableFlags, ImVec2(0, tableHeight))) {
        ImGui::TableSetupColumn("ID", ImGuiTableColumnFlags_WidthStretch);
        ImGui::TableSetupColumn("Count", ImGuiTableColumnFlags_WidthFixed, 60);
        ImGui::TableSetupColumn("Actions", ImGuiTableColumnFlags_WidthFixed, 100);
        ImGui::TableSetupScrollFreeze(0, 1);
        ImGui::TableHeadersRow();

        for (const auto& info : m_allIds) {
            ImGui::TableNextRow();

            ImGui::TableNextColumn();
            ImGui::TextColored(ImVec4(0.73f, 0.60f, 0.97f, 1.0f), "%s", info.id.c_str());

            ImGui::TableNextColumn();
            ImGui::Text("%d", info.paramsCount > 0 ? info.paramsCount : 1);

            ImGui::TableNextColumn();
            ImGui::PushID(info.id.c_str());

            if (ImGui::SmallButton("View")) {
                strncpy_s(m_searchId, info.id.c_str(), sizeof(m_searchId) - 1);
                searchById();
                m_allIds.clear();
            }
            ImGui::SameLine();
            if (ImGui::SmallButton("HTML")) {
                exportHtml(info.id);
            }

            ImGui::PopID();
        }

        ImGui::EndTable();
    }

    ImGui::Spacing();
    if (ImGui::Button("Clear List")) {
        m_allIds.clear();
    }
}

void MainWindow::renderConnectionPanel() {
    ImGui::SetNextWindowSize(ImVec2(550, 450), ImGuiCond_FirstUseEver);
    ImGui::SetNextWindowPos(ImGui::GetMainViewport()->GetCenter(), ImGuiCond_FirstUseEver, ImVec2(0.5f, 0.5f));

    if (ImGui::Begin("Database Connections", &m_showConnectionPanel, ImGuiWindowFlags_NoCollapse)) {
        float panelWidth = ImGui::GetContentRegionAvail().x;

        // Left side: Connection list
        ImGui::BeginChild("ConnectionList", ImVec2(180, -40), true);
        ImGui::TextColored(ImVec4(0.48f, 0.64f, 0.97f, 1.0f), "Saved Connections");
        ImGui::Separator();

        for (int i = 0; i < static_cast<int>(m_config.connections.size()); i++) {
            const auto& conn = m_config.connections[i];
            bool isActive = (i == m_config.activeConnectionIndex && m_sqlConnector.isConnected());
            bool isSelected = (i == m_editingConnectionIndex);

            ImGui::PushID(i);

            // Highlight active connection
            if (isActive) {
                ImGui::PushStyleColor(ImGuiCol_Text, ImVec4(0.62f, 0.81f, 0.42f, 1.0f));
            }

            std::string label = conn.name;
            if (isActive) label += " *";

            if (ImGui::Selectable(label.c_str(), isSelected)) {
                editConnection(i);
            }

            if (isActive) {
                ImGui::PopStyleColor();
            }

            ImGui::PopID();
        }

        ImGui::EndChild();

        ImGui::SameLine();

        // Right side: Connection form
        ImGui::BeginChild("ConnectionForm", ImVec2(0, -40), true);

        if (m_editingConnectionIndex >= 0 || m_connName[0] != '\0') {
            ImGui::TextColored(ImVec4(0.48f, 0.64f, 0.97f, 1.0f),
                m_editingConnectionIndex >= 0 ? "Edit Connection" : "New Connection");
        } else {
            ImGui::TextColored(ImVec4(0.48f, 0.64f, 0.97f, 1.0f), "Connection Details");
        }
        ImGui::Separator();
        ImGui::Spacing();

        ImGui::Text("Name:");
        ImGui::SameLine(100);
        ImGui::SetNextItemWidth(-1);
        ImGui::InputText("##connname", m_connName, sizeof(m_connName));

        ImGui::Text("Server:");
        ImGui::SameLine(100);
        ImGui::SetNextItemWidth(-1);
        ImGui::InputText("##server", m_sqlServer, sizeof(m_sqlServer));

        ImGui::Text("Database:");
        ImGui::SameLine(100);
        ImGui::SetNextItemWidth(-1);
        ImGui::InputText("##database", m_sqlDatabase, sizeof(m_sqlDatabase));

        ImGui::Spacing();
        ImGui::Checkbox("Use Windows Authentication", &m_sqlUseWindowsAuth);

        if (!m_sqlUseWindowsAuth) {
            ImGui::Text("Username:");
            ImGui::SameLine(100);
            ImGui::SetNextItemWidth(-1);
            ImGui::InputText("##username", m_sqlUsername, sizeof(m_sqlUsername));

            ImGui::Text("Password:");
            ImGui::SameLine(100);
            ImGui::SetNextItemWidth(-1);
            ImGui::InputText("##password", m_sqlPassword, sizeof(m_sqlPassword), ImGuiInputTextFlags_Password);
        }

        ImGui::Spacing();
        ImGui::Separator();
        ImGui::Spacing();

        // Status
        if (m_sqlConnector.isConnected()) {
            int activeIdx = m_config.activeConnectionIndex;
            if (activeIdx >= 0 && activeIdx < static_cast<int>(m_config.connections.size())) {
                ImGui::TextColored(ImVec4(0.62f, 0.81f, 0.42f, 1.0f),
                    "Connected: %s", m_config.connections[activeIdx].name.c_str());
            } else {
                ImGui::TextColored(ImVec4(0.62f, 0.81f, 0.42f, 1.0f), "Connected");
            }
        } else {
            ImGui::TextDisabled("Not connected");
        }

        ImGui::Spacing();

        // Calculate button widths based on text
        const ImGuiStyle& style = ImGui::GetStyle();
        float btnPadding = style.FramePadding.x * 2 + 16.0f;
        float saveWidth = ImGui::CalcTextSize("Save").x + btnPadding;
        float connectWidth = ImGui::CalcTextSize("Connect").x + btnPadding;
        float deleteWidth = ImGui::CalcTextSize("Delete").x + btnPadding;
        float clearWidth = ImGui::CalcTextSize("Clear").x + btnPadding;

        // Form buttons
        if (ImGui::Button("Save", ImVec2(saveWidth, 0))) {
            saveCurrentConnection();
        }

        ImGui::SameLine();
        if (m_editingConnectionIndex >= 0) {
            if (ImGui::Button("Connect", ImVec2(connectWidth, 0))) {
                saveCurrentConnection();
                connectToDatabase(static_cast<int>(m_config.connections.size()) - 1);
            }
        }

        ImGui::SameLine();
        if (m_editingConnectionIndex >= 0) {
            if (ImGui::Button("Delete", ImVec2(deleteWidth, 0))) {
                deleteConnection(m_editingConnectionIndex);
                clearConnectionForm();
            }
        }

        ImGui::SameLine();
        if (ImGui::Button("Clear", ImVec2(clearWidth, 0))) {
            clearConnectionForm();
        }

        ImGui::EndChild();

        // Bottom buttons - calculate widths
        float newConnWidth = ImGui::CalcTextSize("+ New Connection").x + btnPadding;
        float disconnectWidth = ImGui::CalcTextSize("Disconnect").x + btnPadding;
        float closeWidth = ImGui::CalcTextSize("Close").x + btnPadding;

        ImGui::Separator();
        ImGui::Spacing();

        if (ImGui::Button("+ New Connection", ImVec2(newConnWidth, 0))) {
            addNewConnection();
        }

        ImGui::SameLine();
        if (m_sqlConnector.isConnected()) {
            if (ImGui::Button("Disconnect", ImVec2(disconnectWidth, 0))) {
                disconnectFromDatabase();
            }
        } else if (m_editingConnectionIndex >= 0) {
            if (ImGui::Button("Connect", ImVec2(connectWidth, 0))) {
                connectToDatabase(m_editingConnectionIndex);
            }
        }

        ImGui::SameLine();
        float closeButtonX = panelWidth - closeWidth;
        ImGui::SetCursorPosX(closeButtonX);
        if (ImGui::Button("Close", ImVec2(closeWidth, 0))) {
            m_showConnectionPanel = false;
        }
    }
    ImGui::End();
}

void MainWindow::renderQueryResultPanel() {
    if (!m_queryResult.success && m_queryResult.error.empty()) {
        ImGui::TextDisabled("No query executed yet.");
        ImGui::TextDisabled("Click 'Execute Query' to run the current SQL.");
        return;
    }

    if (!m_queryResult.error.empty()) {
        ImGui::TextColored(ImVec4(1.0f, 0.4f, 0.4f, 1.0f), "Error:");
        ImGui::TextWrapped("%s", m_queryResult.error.c_str());
        return;
    }

    // Show row count
    if (m_queryResult.columns.empty()) {
        ImGui::TextColored(ImVec4(0.62f, 0.81f, 0.42f, 1.0f),
                          "Query executed. Rows affected: %d", m_queryResult.rowsAffected);
        return;
    }

    ImGui::Text("Rows: %zu | Columns: %zu",
               m_queryResult.rows.size(), m_queryResult.columns.size());

    ImGui::Spacing();

    // Result table
    float tableHeight = ImGui::GetContentRegionAvail().y - 5.0f;

    ImGuiTableFlags tableFlags = ImGuiTableFlags_Borders |
                                  ImGuiTableFlags_RowBg |
                                  ImGuiTableFlags_ScrollX |
                                  ImGuiTableFlags_ScrollY |
                                  ImGuiTableFlags_Resizable;

    int numCols = static_cast<int>(m_queryResult.columns.size());

    if (ImGui::BeginTable("ResultTable", numCols, tableFlags, ImVec2(0, tableHeight))) {
        // Setup columns
        for (const auto& col : m_queryResult.columns) {
            ImGui::TableSetupColumn(col.name.c_str(), ImGuiTableColumnFlags_WidthStretch);
        }
        ImGui::TableSetupScrollFreeze(0, 1);
        ImGui::TableHeadersRow();

        // Data rows
        ImGuiListClipper clipper;
        clipper.Begin(static_cast<int>(m_queryResult.rows.size()));

        while (clipper.Step()) {
            for (int row = clipper.DisplayStart; row < clipper.DisplayEnd; row++) {
                ImGui::TableNextRow();
                const auto& rowData = m_queryResult.rows[row];

                for (size_t col = 0; col < rowData.size(); col++) {
                    ImGui::TableNextColumn();

                    // Highlight NULL values
                    if (rowData[col] == "NULL") {
                        ImGui::TextDisabled("NULL");
                    } else {
                        ImGui::TextUnformatted(rowData[col].c_str());
                    }
                }
            }
        }

        ImGui::EndTable();
    }
}

void MainWindow::renderStatusBar() {
    ImGui::Separator();
    ImGui::Spacing();

    if (m_statusIsError) {
        ImGui::TextColored(ImVec4(1.0f, 0.4f, 0.4f, 1.0f), "[!] %s", m_statusMessage.c_str());
    } else if (!m_statusMessage.empty()) {
        ImGui::TextColored(ImVec4(0.62f, 0.81f, 0.42f, 1.0f), "[OK] %s", m_statusMessage.c_str());
    } else {
        ImGui::TextDisabled("Ready");
    }
}

// Actions
void MainWindow::searchById() {
    if (strlen(m_searchId) == 0) {
        setStatus("Please enter an ID", true);
        return;
    }

    if (!FileHelper::fileExists(m_config.logFilePath)) {
        setStatus("Log file not found: " + m_config.logFilePath, true);
        return;
    }

    setLoading(true, "Searching for ID...");
    m_lastResult = m_processor.processQuery(m_searchId, m_config.logFilePath, m_config.autoCopy);
    m_allIds.clear();
    m_queryResult = SqlResult{}; // Clear previous result
    setLoading(false);

    if (m_lastResult.query.found) {
        if (m_lastResult.copiedToClipboard) {
            setStatus("Found! Copied to clipboard.");
        } else {
            setStatus("Found!");
        }
    } else {
        setStatus("ID not found: " + std::string(m_searchId), true);
    }
}

void MainWindow::searchLastQuery() {
    if (!FileHelper::fileExists(m_config.logFilePath)) {
        setStatus("Log file not found: " + m_config.logFilePath, true);
        return;
    }

    setLoading(true, "Finding last query...");
    m_lastResult = m_processor.processLastQuery(m_config.logFilePath, m_config.autoCopy);
    m_allIds.clear();
    m_queryResult = SqlResult{};
    setLoading(false);

    if (m_lastResult.query.found) {
        strncpy_s(m_searchId, m_lastResult.query.id.c_str(), sizeof(m_searchId) - 1);
        if (m_lastResult.copiedToClipboard) {
            setStatus("Last query found! Copied to clipboard.");
        } else {
            setStatus("Last query found!");
        }
    } else {
        setStatus("No SQL queries found in log file", true);
    }
}

void MainWindow::loadAllIds() {
    if (!FileHelper::fileExists(m_config.logFilePath)) {
        setStatus("Log file not found: " + m_config.logFilePath, true);
        return;
    }

    setLoading(true, "Loading all IDs...");
    m_allIds = m_parser.getAllIds(m_config.logFilePath);
    m_lastResult = ProcessResult{};
    setLoading(false);

    if (m_allIds.empty()) {
        setStatus("No IDs found in log file", true);
    } else {
        setStatus("Found " + std::to_string(m_allIds.size()) + " IDs");
    }
}

void MainWindow::exportHtml(const std::string& targetId) {
    if (!FileHelper::fileExists(m_config.logFilePath)) {
        setStatus("Log file not found", true);
        return;
    }

    setLoading(true, "Exporting HTML for " + targetId + "...");
    auto executions = m_parser.parseLogFileAdvanced(m_config.logFilePath, targetId);

    if (executions.empty()) {
        setLoading(false);
        setStatus("No data found for ID: " + targetId, true);
        return;
    }

    HtmlOptions options;
    options.title = "ID: " + targetId;
    options.logFile = m_config.logFilePath;

    std::string html = m_htmlGenerator.generateReport(executions, options);
    std::string outputPath = m_config.htmlOutputPath + "\\sql_report_" + targetId + ".html";

    if (m_htmlGenerator.saveReport(html, outputPath)) {
        setLoading(false);
        setStatus("HTML exported: " + outputPath);
        ShellExecuteA(nullptr, "open", outputPath.c_str(), nullptr, nullptr, SW_SHOWNORMAL);
    } else {
        setLoading(false);
        setStatus("Failed to export HTML", true);
    }
}

void MainWindow::exportHtmlAll() {
    if (!FileHelper::fileExists(m_config.logFilePath)) {
        setStatus("Log file not found", true);
        return;
    }

    setLoading(true, "Collecting all IDs...");
    auto ids = m_parser.getAllIds(m_config.logFilePath);

    if (ids.empty()) {
        setLoading(false);
        setStatus("No IDs found in log file", true);
        return;
    }

    setLoading(true, "Exporting HTML for all queries...");
    std::vector<Execution> allExecutions;
    for (const auto& info : ids) {
        auto executions = m_parser.parseLogFileAdvanced(m_config.logFilePath, info.id);
        allExecutions.insert(allExecutions.end(), executions.begin(), executions.end());
    }

    HtmlOptions options;
    options.title = "All SQL Queries";
    options.logFile = m_config.logFilePath;

    std::string html = m_htmlGenerator.generateReport(allExecutions, options);
    std::string outputPath = m_config.htmlOutputPath + "\\sql_report_all.html";

    if (m_htmlGenerator.saveReport(html, outputPath)) {
        setLoading(false);
        setStatus("HTML exported: " + outputPath + " (" +
                  std::to_string(allExecutions.size()) + " queries)");
        ShellExecuteA(nullptr, "open", outputPath.c_str(), nullptr, nullptr, SW_SHOWNORMAL);
    } else {
        setLoading(false);
        setStatus("Failed to export HTML", true);
    }
}

void MainWindow::copyToClipboard() {
    std::string sql = m_lastResult.filledSql.empty() ?
                      m_lastResult.query.sql : m_lastResult.filledSql;

    if (sql.empty()) {
        setStatus("No SQL to copy", true);
        return;
    }

    if (ClipboardHelper::copyToClipboard(sql)) {
        setStatus("Copied to clipboard!");
    } else {
        setStatus("Failed to copy to clipboard", true);
    }
}

void MainWindow::browseLogFile() {
    char filename[MAX_PATH] = "";

    OPENFILENAMEA ofn = {};
    ofn.lStructSize = sizeof(ofn);
    ofn.hwndOwner = nullptr;
    ofn.lpstrFilter = "Log Files\0*.log\0All Files\0*.*\0";
    ofn.lpstrFile = filename;
    ofn.nMaxFile = MAX_PATH;
    ofn.Flags = OFN_FILEMUSTEXIST | OFN_PATHMUSTEXIST;
    ofn.lpstrTitle = "Select Log File";

    if (GetOpenFileNameA(&ofn)) {
        m_config.logFilePath = filename;
        m_configManager.save(m_config);
        setStatus("Log file path updated");
    }
}

void MainWindow::browseOutputPath() {
    char path[MAX_PATH] = "";

    BROWSEINFOA bi = {};
    bi.lpszTitle = "Select HTML Output Folder";
    bi.ulFlags = BIF_RETURNONLYFSDIRS | BIF_NEWDIALOGSTYLE;

    LPITEMIDLIST pidl = SHBrowseForFolderA(&bi);
    if (pidl && SHGetPathFromIDListA(pidl, path)) {
        m_config.htmlOutputPath = path;
        m_configManager.save(m_config);
        setStatus("Output folder updated");
        CoTaskMemFree(pidl);
    }
}

// SQL Actions
void MainWindow::connectToDatabase() {
    // Connect using form data (for quick connect)
    if (strlen(m_sqlServer) == 0 || strlen(m_sqlDatabase) == 0) {
        setStatus("Please enter server and database name", true);
        return;
    }

    setStatus("Connecting to database...");

    bool success = m_sqlConnector.connect(
        m_sqlServer,
        m_sqlDatabase,
        m_sqlUsername,
        m_sqlPassword,
        m_sqlUseWindowsAuth
    );

    if (success) {
        setStatus("Connected to database successfully!");
    } else {
        setStatus("Connection failed: " + m_sqlConnector.getLastError(), true);
    }
}

void MainWindow::connectToDatabase(int connectionIndex) {
    if (connectionIndex < 0 || connectionIndex >= static_cast<int>(m_config.connections.size())) {
        setStatus("Invalid connection index", true);
        return;
    }

    const auto& conn = m_config.connections[connectionIndex];

    if (conn.server.empty() || conn.database.empty()) {
        setStatus("Server and database name required", true);
        return;
    }

    setStatus("Connecting to " + conn.name + "...");

    bool success = m_sqlConnector.connect(
        conn.server,
        conn.database,
        conn.username,
        conn.password,
        conn.useWindowsAuth
    );

    if (success) {
        m_config.activeConnectionIndex = connectionIndex;
        m_configManager.save(m_config);
        setStatus("Connected to " + conn.name);
    } else {
        setStatus("Connection failed: " + m_sqlConnector.getLastError(), true);
    }
}

void MainWindow::disconnectFromDatabase() {
    m_sqlConnector.disconnect();
    m_config.activeConnectionIndex = -1;
    m_queryResult = SqlResult{};
    setStatus("Disconnected from database");
}

void MainWindow::executeCurrentQuery() {
    if (!m_sqlConnector.isConnected()) {
        setStatus("Not connected to database", true);
        return;
    }

    std::string sql = m_lastResult.filledSql.empty() ?
                      m_lastResult.query.sql : m_lastResult.filledSql;

    if (sql.empty()) {
        setStatus("No SQL query to execute", true);
        return;
    }

    setStatus("Executing query...");

    m_queryResult = m_sqlConnector.executeQuery(sql);

    if (m_queryResult.success) {
        if (m_queryResult.columns.empty()) {
            setStatus("Query executed. Rows affected: " + std::to_string(m_queryResult.rowsAffected));
        } else {
            setStatus("Query returned " + std::to_string(m_queryResult.rows.size()) + " rows");
        }
    } else {
        setStatus("Query failed: " + m_queryResult.error, true);
    }
}

void MainWindow::copyResultAsCsv() {
    if (!m_queryResult.success || m_queryResult.columns.empty()) {
        setStatus("No result to copy", true);
        return;
    }

    std::string csv = SqlConnector::resultToCsv(m_queryResult, m_csvSeparator);

    if (ClipboardHelper::copyToClipboard(csv)) {
        setStatus("Result copied as CSV (" + std::to_string(m_queryResult.rows.size()) + " rows)");
    } else {
        setStatus("Failed to copy to clipboard", true);
    }
}
