#include "SqlConnector.h"

#define WIN32_LEAN_AND_MEAN
#include <windows.h>
#include <sql.h>
#include <sqlext.h>
#include <sstream>

SqlConnector::SqlConnector() {}

SqlConnector::~SqlConnector() {
    disconnect();
}

bool SqlConnector::connect(const std::string& server, const std::string& database,
                           const std::string& username, const std::string& password,
                           bool useWindowsAuth) {
    disconnect();

    SQLRETURN ret;

    // Allocate environment handle
    ret = SQLAllocHandle(SQL_HANDLE_ENV, SQL_NULL_HANDLE, &m_hEnv);
    if (!SQL_SUCCEEDED(ret)) {
        m_lastError = "Failed to allocate environment handle";
        return false;
    }

    // Set ODBC version
    ret = SQLSetEnvAttr(m_hEnv, SQL_ATTR_ODBC_VERSION, (void*)SQL_OV_ODBC3, 0);
    if (!SQL_SUCCEEDED(ret)) {
        extractError(m_hEnv, SQL_HANDLE_ENV);
        SQLFreeHandle(SQL_HANDLE_ENV, m_hEnv);
        m_hEnv = nullptr;
        return false;
    }

    // Allocate connection handle
    ret = SQLAllocHandle(SQL_HANDLE_DBC, m_hEnv, &m_hDbc);
    if (!SQL_SUCCEEDED(ret)) {
        extractError(m_hEnv, SQL_HANDLE_ENV);
        SQLFreeHandle(SQL_HANDLE_ENV, m_hEnv);
        m_hEnv = nullptr;
        return false;
    }

    // Build connection string
    std::string connStr;
    if (useWindowsAuth) {
        connStr = "DRIVER={ODBC Driver 17 for SQL Server};"
                  "SERVER=" + server + ";"
                  "DATABASE=" + database + ";"
                  "Trusted_Connection=yes;";
    } else {
        connStr = "DRIVER={ODBC Driver 17 for SQL Server};"
                  "SERVER=" + server + ";"
                  "DATABASE=" + database + ";"
                  "UID=" + username + ";"
                  "PWD=" + password + ";";
    }

    // Try connecting with different drivers if first fails
    std::vector<std::string> drivers = {
        "ODBC Driver 18 for SQL Server",
        "ODBC Driver 17 for SQL Server",
        "SQL Server Native Client 11.0",
        "SQL Server"
    };

    bool connected = false;
    for (const auto& driver : drivers) {
        std::string tryConnStr;
        if (useWindowsAuth) {
            tryConnStr = "DRIVER={" + driver + "};"
                        "SERVER=" + server + ";"
                        "DATABASE=" + database + ";"
                        "Trusted_Connection=yes;"
                        "TrustServerCertificate=yes;";
        } else {
            tryConnStr = "DRIVER={" + driver + "};"
                        "SERVER=" + server + ";"
                        "DATABASE=" + database + ";"
                        "UID=" + username + ";"
                        "PWD=" + password + ";"
                        "TrustServerCertificate=yes;";
        }

        SQLCHAR outConnStr[1024];
        SQLSMALLINT outConnStrLen;

        ret = SQLDriverConnectA(
            m_hDbc,
            NULL,
            (SQLCHAR*)tryConnStr.c_str(),
            SQL_NTS,
            outConnStr,
            sizeof(outConnStr),
            &outConnStrLen,
            SQL_DRIVER_NOPROMPT
        );

        if (SQL_SUCCEEDED(ret)) {
            connected = true;
            break;
        }
    }

    if (!connected) {
        extractError(m_hDbc, SQL_HANDLE_DBC);
        SQLFreeHandle(SQL_HANDLE_DBC, m_hDbc);
        SQLFreeHandle(SQL_HANDLE_ENV, m_hEnv);
        m_hDbc = nullptr;
        m_hEnv = nullptr;
        return false;
    }

    m_connected = true;
    return true;
}

void SqlConnector::disconnect() {
    if (m_hDbc) {
        if (m_connected) {
            SQLDisconnect(m_hDbc);
        }
        SQLFreeHandle(SQL_HANDLE_DBC, m_hDbc);
        m_hDbc = nullptr;
    }
    if (m_hEnv) {
        SQLFreeHandle(SQL_HANDLE_ENV, m_hEnv);
        m_hEnv = nullptr;
    }
    m_connected = false;
}

bool SqlConnector::isConnected() const {
    return m_connected;
}

std::string SqlConnector::getLastError() const {
    return m_lastError;
}

SqlResult SqlConnector::executeQuery(const std::string& sql) {
    SqlResult result;

    if (!m_connected) {
        result.error = "Not connected to database";
        return result;
    }

    SQLHSTMT hStmt = nullptr;
    SQLRETURN ret;

    // Allocate statement handle
    ret = SQLAllocHandle(SQL_HANDLE_STMT, m_hDbc, &hStmt);
    if (!SQL_SUCCEEDED(ret)) {
        extractError(m_hDbc, SQL_HANDLE_DBC);
        result.error = m_lastError;
        return result;
    }

    // Execute query
    ret = SQLExecDirectA(hStmt, (SQLCHAR*)sql.c_str(), SQL_NTS);
    if (!SQL_SUCCEEDED(ret)) {
        extractError(hStmt, SQL_HANDLE_STMT);
        result.error = m_lastError;
        SQLFreeHandle(SQL_HANDLE_STMT, hStmt);
        return result;
    }

    // Get column count
    SQLSMALLINT numCols;
    ret = SQLNumResultCols(hStmt, &numCols);
    if (!SQL_SUCCEEDED(ret)) {
        extractError(hStmt, SQL_HANDLE_STMT);
        result.error = m_lastError;
        SQLFreeHandle(SQL_HANDLE_STMT, hStmt);
        return result;
    }

    // If no columns, it's a non-SELECT query
    if (numCols == 0) {
        SQLLEN rowCount;
        SQLRowCount(hStmt, &rowCount);
        result.rowsAffected = static_cast<int>(rowCount);
        result.success = true;
        SQLFreeHandle(SQL_HANDLE_STMT, hStmt);
        return result;
    }

    // Get column info
    for (SQLSMALLINT i = 1; i <= numCols; i++) {
        SqlColumn col;
        SQLCHAR colName[256];
        SQLSMALLINT colNameLen;
        SQLSMALLINT dataType;
        SQLULEN colSize;
        SQLSMALLINT decimalDigits;
        SQLSMALLINT nullable;

        ret = SQLDescribeColA(hStmt, i, colName, sizeof(colName), &colNameLen,
                              &dataType, &colSize, &decimalDigits, &nullable);
        if (SQL_SUCCEEDED(ret)) {
            col.name = std::string((char*)colName, colNameLen);
            col.type = dataType;
            col.size = static_cast<int>(colSize);
            result.columns.push_back(col);
        }
    }

    // Fetch rows
    while (true) {
        ret = SQLFetch(hStmt);
        if (ret == SQL_NO_DATA) break;
        if (!SQL_SUCCEEDED(ret)) {
            extractError(hStmt, SQL_HANDLE_STMT);
            break;
        }

        std::vector<std::string> row;
        for (SQLSMALLINT i = 1; i <= numCols; i++) {
            SQLCHAR buffer[8192];
            SQLLEN indicator;

            ret = SQLGetData(hStmt, i, SQL_C_CHAR, buffer, sizeof(buffer), &indicator);
            if (SQL_SUCCEEDED(ret)) {
                if (indicator == SQL_NULL_DATA) {
                    row.push_back("NULL");
                } else {
                    row.push_back(std::string((char*)buffer));
                }
            } else {
                row.push_back("");
            }
        }
        result.rows.push_back(row);
    }

    result.success = true;
    SQLFreeHandle(SQL_HANDLE_STMT, hStmt);
    return result;
}

std::string SqlConnector::resultToCsv(const SqlResult& result, const std::string& separator) {
    std::stringstream ss;

    // Header
    for (size_t i = 0; i < result.columns.size(); i++) {
        if (i > 0) ss << separator;

        // Escape if contains separator or quotes
        std::string val = result.columns[i].name;
        bool needQuotes = val.find(separator) != std::string::npos ||
                         val.find('"') != std::string::npos ||
                         val.find('\n') != std::string::npos;
        if (needQuotes) {
            // Escape quotes
            size_t pos = 0;
            while ((pos = val.find('"', pos)) != std::string::npos) {
                val.replace(pos, 1, "\"\"");
                pos += 2;
            }
            ss << "\"" << val << "\"";
        } else {
            ss << val;
        }
    }
    ss << "\n";

    // Data rows
    for (const auto& row : result.rows) {
        for (size_t i = 0; i < row.size(); i++) {
            if (i > 0) ss << separator;

            std::string val = row[i];
            bool needQuotes = val.find(separator) != std::string::npos ||
                             val.find('"') != std::string::npos ||
                             val.find('\n') != std::string::npos;
            if (needQuotes) {
                size_t pos = 0;
                while ((pos = val.find('"', pos)) != std::string::npos) {
                    val.replace(pos, 1, "\"\"");
                    pos += 2;
                }
                ss << "\"" << val << "\"";
            } else {
                ss << val;
            }
        }
        ss << "\n";
    }

    return ss.str();
}

void SqlConnector::extractError(void* handle, int handleType) {
    SQLCHAR sqlState[6];
    SQLINTEGER nativeError;
    SQLCHAR message[1024];
    SQLSMALLINT msgLen;

    SQLRETURN ret = SQLGetDiagRecA(
        handleType, handle, 1, sqlState, &nativeError,
        message, sizeof(message), &msgLen
    );

    if (SQL_SUCCEEDED(ret)) {
        m_lastError = std::string((char*)message, msgLen);
    } else {
        m_lastError = "Unknown database error";
    }
}
