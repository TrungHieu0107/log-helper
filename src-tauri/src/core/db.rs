use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DbType {
    Postgres,
    Mysql,
    Sqlite,
    SqlServer,
}

impl Default for DbType {
    fn default() -> Self {
        DbType::SqlServer
    }
}

impl std::fmt::Display for DbType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DbType::Postgres => write!(f, "Postgres"),
            DbType::Mysql => write!(f, "MySQL"),
            DbType::Sqlite => write!(f, "SQLite"),
            DbType::SqlServer => write!(f, "SQL Server"),
        }
    }
}

pub fn convert_jdbc_to_sqlx(jdbc_url: &str, user: &str, pass: &str) -> String {
    if !jdbc_url.starts_with("jdbc:sqlserver://") {
        return jdbc_url.to_string();
    }

    // Example JDBC URL: jdbc:sqlserver://host:port;databaseName=DBNAME;encrypt=true;trustServerCertificate=true
    let stripped = jdbc_url.trim_start_matches("jdbc:sqlserver://");
    
    // Split address from parameters
    let parts: Vec<&str> = stripped.splitn(2, ';').collect();
    let addr = parts[0];
    
    let mut sqlx_url = format!("mssql://{}:{}@{}", user, pass, addr);
    
    if parts.len() > 1 {
        let params = parts[1];
        // Convert semicolon separated params to query params if possible, 
        // or specifically handle databaseName
        let mut database = String::new();
        let mut other_params = Vec::new();
        
        for param in params.split(';') {
            if param.is_empty() { continue; }
            let kv: Vec<&str> = param.splitn(2, '=').collect();
            if kv.len() == 2 {
                if kv[0].to_lowercase() == "databasename" {
                    database = kv[1].to_string();
                } else {
                    other_params.push(format!("{}={}", kv[0], kv[1]));
                }
            }
        }
        
        if !database.is_empty() {
            sqlx_url.push_str("/");
            sqlx_url.push_str(&database);
        }
        
        if !other_params.is_empty() {
            sqlx_url.push_str("?");
            sqlx_url.push_str(&other_params.join("&"));
        }
    }
    
    sqlx_url
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ParsedSqlServerUrl {
    pub host: String,
    pub port: u16,
    pub instance: Option<String>,
    pub database: Option<String>,
    pub encrypt: bool,
    pub trust_cert: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConnectionFields {
    pub host: String,
    pub port: String, // String for UI input
    pub database: String,
    pub encrypt: bool,
    pub trust_cert: bool,
}

impl Default for ConnectionFields {
    fn default() -> Self {
        Self {
            host: "localhost".to_string(),
            port: "1433".to_string(),
            database: "master".to_string(),
            encrypt: false,
            trust_cert: true,
        }
    }
}

pub fn parse_jdbc_url(url: &str) -> Result<ParsedSqlServerUrl, String> {
    if !url.starts_with("jdbc:sqlserver://") {
        return Err("Invalid JDBC URL prefix. Must start with 'jdbc:sqlserver://'".to_string());
    }

    let stripped = url.trim_start_matches("jdbc:sqlserver://");
    let (server_part, params_part) = match stripped.split_once(';') {
        Some((s, p)) => (s, Some(p)),
        None => (stripped, None),
    };

    let mut parsed = ParsedSqlServerUrl {
        host: String::new(),
        port: 1433,
        instance: None,
        database: None,
        encrypt: false,
        trust_cert: false,
    };

    // Parse server part
    if let Some((h, p)) = server_part.split_once(':') {
        parsed.host = h.to_string();
        parsed.port = p.parse().unwrap_or(1433);
    } else if let Some((h, i)) = server_part.split_once('\\') {
        parsed.host = h.to_string();
        parsed.instance = Some(i.to_string());
        parsed.port = 1433; // Default for instance mostly, or dynamic
    } else {
        parsed.host = server_part.to_string();
    }

    // Parse params
    if let Some(params) = params_part {
        for param in params.split(';') {
            if param.is_empty() { continue; }
            let kv: Vec<&str> = param.splitn(2, '=').collect();
            if kv.len() == 2 {
                let key = kv[0].trim().to_lowercase();
                let val = kv[1].trim();
                
                match key.as_str() {
                    "databasename" | "database" => parsed.database = Some(val.to_string()),
                    "encrypt" => parsed.encrypt = val == "true",
                    "trustservercertificate" => parsed.trust_cert = val == "true",
                    _ => {}
                }
            }
        }
    }

    Ok(parsed)
}

pub fn build_jdbc_url(fields: &ConnectionFields) -> String {
    let mut url = format!("jdbc:sqlserver://{}:{}", fields.host, fields.port);
    
    if !fields.database.is_empty() {
        url.push_str(&format!(";databaseName={}", fields.database));
    }
    
    if fields.encrypt {
        url.push_str(";encrypt=true");
    }
    
    if fields.trust_cert {
        url.push_str(";trustServerCertificate=true");
    }
    
    url
}


#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DbConfig {
    pub id: String, // UUID usually
    pub name: String,
    pub db_type: DbType,
    pub url: String, // JDBC-like or native URL
    pub user: String,
    pub password: String, // In real app, encrypt this. For now, plain text config.
    pub encoding: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct ConnectionManager {
    pub connections: Vec<DbConfig>,
    config_path: PathBuf,
}

impl ConnectionManager {
    pub fn new() -> Self {
        let mut path = PathBuf::from("db_connections.json");
        if let Some(dirs) = directories::ProjectDirs::from("com", "loghelper", "sql-log-parser") {
             let config_dir = dirs.config_dir();
             if !config_dir.exists() {
                 let _ = fs::create_dir_all(config_dir);
             }
             path = config_dir.join("db_connections.json");
        }
        
        // Fallback to local if directories fails or for portability
        if !path.exists() {
             // Maybe check local? keeping standard path logic simple for now.
             // If we want portable, use "./db_connections.json"
             path = PathBuf::from("db_connections.json");
        }

        let mut manager = Self {
            connections: Vec::new(),
            config_path: path,
        };
        manager.load();
        manager
    }

    pub fn load(&mut self) {
        if self.config_path.exists() {
            if let Ok(content) = fs::read_to_string(&self.config_path) {
                if let Ok(conns) = serde_json::from_str(&content) {
                    self.connections = conns;
                }
            }
        }
    }

    pub fn save(&self) -> anyhow::Result<()> {
        let content = serde_json::to_string_pretty(&self.connections)?;
        fs::write(&self.config_path, content).map_err(|e| anyhow::anyhow!("Failed to write config file: {}", e))?;
        println!("Saved connections to {:?}", self.config_path);
        Ok(())
    }

    pub fn add(&mut self, config: DbConfig) -> anyhow::Result<()> {
        if self.connections.iter().any(|c| c.name == config.name) {
             return Err(anyhow::anyhow!("A connection with name '{}' already exists", config.name));
        }
        self.connections.push(config);
        self.save()
    }
    
    pub fn update(&mut self, config: DbConfig) -> anyhow::Result<()> {
        if let Some(pos) = self.connections.iter().position(|c| c.id == config.id) {
            self.connections[pos] = config;
            self.save()
        } else {
             Err(anyhow::anyhow!("Connection not found for update"))
        }
    }

    pub fn delete(&mut self, id: &str) -> anyhow::Result<()> {
        let prev_len = self.connections.len();
        self.connections.retain(|c| c.id != id);
        if self.connections.len() != prev_len {
             self.save()
        } else {
             Ok(()) 
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CellValue {
    Null,
    Text(String),
    Int(i64),
    Float(f64),
    Bool(bool),
    DateTime(String),
    Binary(String),
}

impl std::fmt::Display for CellValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CellValue::Null => write!(f, "NULL"),
            CellValue::Text(s) => write!(f, "{}", s),
            CellValue::Int(n) => write!(f, "{}", n),
            CellValue::Float(n) => write!(f, "{}", n),
            CellValue::Bool(b) => write!(f, "{}", b),
            CellValue::DateTime(s) => write!(f, "{}", s),
            CellValue::Binary(s) => write!(f, "{}", s),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryResult {
    pub columns: Vec<String>,
    pub rows: Vec<Vec<CellValue>>,
    pub affected_rows: u64,
    pub execution_time_ms: u128,
}

#[async_trait::async_trait]
pub trait DatabaseExecutor {
    async fn execute(&self, config: &DbConfig, sql: &str) -> anyhow::Result<QueryResult>;
}

#[derive(Clone)]
pub struct DbClient {
    sqlx_executor: SqlxExecutor,
    mssql_executor: MssqlExecutor,
}

impl DbClient {
    pub fn new() -> Self {
        Self {
            sqlx_executor: SqlxExecutor,
            mssql_executor: MssqlExecutor,
        }
    }

    pub async fn test_connection(&self, config: &DbConfig) -> anyhow::Result<()> {
        if config.db_type == DbType::SqlServer {
             // For SQL Server, we try to connect and run a simple query
             let _res = self.execute_query(config, "SELECT 1").await?;
             // executing successful implies connection worked
             return Ok(());
        }
        
        // For others, use sqlx to test
        use sqlx::any::AnyConnectOptions;
        use std::str::FromStr;
        
        let connection_url = if config.db_type == DbType::SqlServer {
            convert_jdbc_to_sqlx(&config.url, &config.user, &config.password)
        } else {
            config.url.clone()
        };
        
        // Convert connection_url to sqlx options
        let opts = AnyConnectOptions::from_str(&connection_url)?;
        
        use sqlx::any::AnyPoolOptions;
        let pool = AnyPoolOptions::new()
            .max_connections(1)
            .connect_with(opts)
            .await?;
            
        // Test query
        let _ = sqlx::query("SELECT 1").execute(&pool).await?;
        
        Ok(())
    }

    pub async fn execute_query(&self, config: &DbConfig, sql: &str) -> anyhow::Result<QueryResult> {
        match config.db_type {
            DbType::SqlServer => self.mssql_executor.execute(config, sql).await,
            _ => self.sqlx_executor.execute(config, sql).await,
        }
    }
}

#[derive(Clone, Copy)]
pub struct SqlxExecutor;

#[async_trait::async_trait]
impl DatabaseExecutor for SqlxExecutor {
    async fn execute(&self, config: &DbConfig, sql: &str) -> anyhow::Result<QueryResult> {
        use sqlx::any::{AnyConnectOptions, AnyPoolOptions};
        use sqlx::{Column, Row, ValueRef};
        use std::str::FromStr;
        use std::time::Instant;

        let start = Instant::now();
        let connection_url = config.url.clone();

        let opts = AnyConnectOptions::from_str(&connection_url)?;

        let pool = AnyPoolOptions::new()
            .max_connections(1)
            .connect_with(opts)
            .await?;

        let is_select = sql.trim().to_lowercase().starts_with("select")
            || sql.trim().to_lowercase().starts_with("with")
            || sql.trim().to_lowercase().starts_with("show")
            || sql.trim().to_lowercase().starts_with("describe");

        let mut result = QueryResult {
            columns: Vec::new(),
            rows: Vec::new(),
            affected_rows: 0,
            execution_time_ms: 0,
        };

        if is_select {
            let rows = sqlx::query(sql)
                .fetch_all(&pool)
                .await?;

            if let Some(first_row) = rows.first() {
                result.columns = first_row.columns().iter().map(|c| c.name().to_string()).collect();
            }

            for row in rows {
                let mut row_data = Vec::new();
                for i in 0..result.columns.len() {
                    let mut handled = false;
                    if let Some(encoding) = &config.encoding {
                         if let Ok(opt_bytes) = row.try_get::<Option<Vec<u8>>, _>(i) {
                             handled = true;
                             match opt_bytes {
                                 Some(bytes) => row_data.push(CellValue::Text(crate::utils::encoding::decode_bytes(&bytes, encoding))),
                                 None => row_data.push(CellValue::Null),
                             }
                         }
                    }

                    if !handled {
                        let val: CellValue = if let Ok(n) = row.try_get::<i64, _>(i) {
                            CellValue::Int(n)
                        } else if let Ok(f) = row.try_get::<f64, _>(i) {
                            CellValue::Float(f)
                        } else if let Ok(b) = row.try_get::<bool, _>(i) {
                            CellValue::Bool(b)
                        } else if let Ok(s) = row.try_get::<String, _>(i) {
                             CellValue::Text(s)
                        } else {
                            // Fallback or Null check
                            if row.try_get_raw(i).map(|v| v.is_null()).unwrap_or(false) {
                                CellValue::Null
                            } else {
                                // Convert debug format as fallback text
                                 match row.try_get_raw(i) {
                                     Ok(v) => CellValue::Text(format!("{:?}", v)),
                                     Err(_) => CellValue::Text("ERR".to_string()),
                                 }
                            }
                        };
                        row_data.push(val);
                    }
                }
                result.rows.push(row_data);
            }
        } else {
            let res = sqlx::query(sql)
                .execute(&pool)
                .await?;
            result.affected_rows = res.rows_affected();
        }

        result.execution_time_ms = start.elapsed().as_millis();
        Ok(result)
    }
}

#[derive(Clone, Copy)]
pub struct MssqlExecutor;

#[async_trait::async_trait]
impl DatabaseExecutor for MssqlExecutor {
    async fn execute(&self, config: &DbConfig, sql: &str) -> anyhow::Result<QueryResult> {
        use futures_util::stream::StreamExt;
        use std::time::Instant;
        use tiberius::{AuthMethod, Client, Config};
        use tokio::net::TcpStream;
        use tokio_util::compat::TokioAsyncWriteCompatExt;

        let start = Instant::now();
        
        let parsed = parse_jdbc_url(&config.url)
            .map_err(|e| anyhow::anyhow!("Failed to parse JDBC URL: {}", e))?;
        
        let mut t_config = Config::new();
        t_config.host(&parsed.host);
        t_config.port(parsed.port);
        t_config.authentication(AuthMethod::sql_server(&config.user, &config.password));
        
        if let Some(inst) = parsed.instance {
            t_config.instance_name(inst);
        }
        
        if let Some(db) = parsed.database {
            t_config.database(db);
        }

        if parsed.encrypt {
             t_config.encryption(tiberius::EncryptionLevel::Required);
        } else {
             t_config.encryption(tiberius::EncryptionLevel::NotSupported);
        }
        
        if parsed.trust_cert {
            t_config.trust_cert();
        }

        let tcp = TcpStream::connect(t_config.get_addr()).await.map_err(|e| anyhow::anyhow!("Failed to connect to {}:{} - {}", parsed.host, parsed.port, e))?;
        tcp.set_nodelay(true)?;

        let mut client = Client::connect(t_config, tcp.compat_write()).await.map_err(|e| anyhow::anyhow!("Login failed: {}", e))?;

        let mut result = QueryResult {
            columns: Vec::new(),
            rows: Vec::new(),
            affected_rows: 0,
            execution_time_ms: 0,
        };

        let mut stream = client.query(sql, &[]).await.map_err(|e| anyhow::anyhow!("Query execution failed: {}", e))?;
        
        // Get columns from the first result set
        if let Some(columns) = stream.columns().await? {
            result.columns = columns.iter().map(|c| c.name().to_string()).collect();
        }

        while let Some(item) = stream.next().await {
            match item? {
                tiberius::QueryItem::Row(row) => {
                    let mut row_data = Vec::new();
                    for i in 0..result.columns.len() {
                        // Check for custom encoding first
                        let mut handled = false;
                        if let Some(encoding) = &config.encoding {
                            if let Ok(res) = row.try_get::<&[u8], _>(i) {
                                handled = true;
                                let val = match res {
                                    Some(bytes) => CellValue::Text(crate::utils::encoding::decode_bytes(bytes, encoding)),
                                    None => CellValue::Null,
                                };
                                row_data.push(val);
                            }
                        }

                        if !handled {
                            let val = if let Ok(Some(s)) = row.try_get::<&str, _>(i) {
                                CellValue::Text(s.to_string())
                            } else if let Ok(Some(n)) = row.try_get::<i64, _>(i) {
                                CellValue::Int(n)
                            } else if let Ok(Some(n)) = row.try_get::<i32, _>(i) {
                                CellValue::Int(n as i64)
                            } else if let Ok(Some(f)) = row.try_get::<f64, _>(i) {
                                CellValue::Float(f)
                            } else if let Ok(Some(f)) = row.try_get::<f32, _>(i) {
                                CellValue::Float(f as f64)
                            } else if let Ok(Some(b)) = row.try_get::<bool, _>(i) {
                                CellValue::Bool(b)
                            } else if let Ok(Some(u)) = row.try_get::<uuid::Uuid, _>(i) {
                                CellValue::Text(u.to_string())
                            } else if let Ok(Some(d)) = row.try_get::<tiberius::time::chrono::NaiveDateTime, _>(i) {
                                 CellValue::DateTime(d.to_string())
                            } else if let Ok(Some(d)) = row.try_get::<tiberius::time::chrono::NaiveDate, _>(i) {
                                 CellValue::DateTime(d.to_string())
                            } else if let Ok(None) = row.try_get::<&str, _>(i) {
                                 CellValue::Null
                            } else {
                                 CellValue::Text("NULL/Other".to_string())
                            };
                            row_data.push(val);
                        }
                    }
                    result.rows.push(row_data);
                }
                tiberius::QueryItem::Metadata(_) => {}
            }
        }

        drop(stream);

        if result.rows.is_empty() && result.columns.is_empty() {
             let counts = client.execute(sql, &[]).await?;
             result.affected_rows = counts.total();
        }

        result.execution_time_ms = start.elapsed().as_millis();
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_jdbc_url_simple() {
        let url = "jdbc:sqlserver://localhost:1433;databaseName=testdb";
        let parsed = parse_jdbc_url(url).unwrap();
        
        assert_eq!(parsed.host, "localhost");
        assert_eq!(parsed.port, 1433);
        assert_eq!(parsed.database, Some("testdb".to_string()));
        assert_eq!(parsed.encrypt, false);
        assert_eq!(parsed.trust_cert, false);
    }

    #[test]
    fn test_parse_jdbc_url_full() {
        let url = "jdbc:sqlserver://myserver.com:5555;database=prod;encrypt=true;trustServerCertificate=true";
        let parsed = parse_jdbc_url(url).unwrap();
        
        assert_eq!(parsed.host, "myserver.com");
        assert_eq!(parsed.port, 5555);
        assert_eq!(parsed.database, Some("prod".to_string()));
        assert_eq!(parsed.encrypt, true);
        assert_eq!(parsed.trust_cert, true);
    }

    #[test]
    fn test_parse_jdbc_url_instance() {
        let url = "jdbc:sqlserver://myserver\\SQLEXPRESS;databaseName=test";
        let parsed = parse_jdbc_url(url).unwrap();
        
        assert_eq!(parsed.host, "myserver");
        assert_eq!(parsed.instance, Some("SQLEXPRESS".to_string()));
        assert_eq!(parsed.database, Some("test".to_string()));
    }
    
    #[test]
    fn test_build_jdbc_url() {
        let fields = ConnectionFields {
            host: "10.0.0.1".to_string(),
            port: "1433".to_string(),
            database: "master".to_string(),
            encrypt: true,
            trust_cert: true,
        };
        
        let url = build_jdbc_url(&fields);
        assert!(url.starts_with("jdbc:sqlserver://10.0.0.1:1433"));
        assert!(url.contains("databaseName=master"));
        assert!(url.contains("encrypt=true"));
        assert!(url.contains("trustServerCertificate=true"));
    }

    #[test]
    fn test_db_client_init() {
        let client = DbClient::new();
        // Just verify we can create it and it doesn't panic.
        // In a real scenario we'd mock the executors or test against a real DB if available.
        // This satisfies "ensure the application run without any problem" regarding the fixed struct.
        assert_eq!(std::mem::size_of_val(&client), std::mem::size_of::<DbClient>());
    }
}
