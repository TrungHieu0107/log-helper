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

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DbConfig {
    pub id: String, // UUID usually
    pub name: String,
    pub db_type: DbType,
    pub url: String, // JDBC-like or native URL
    pub user: String,
    pub password: String, // In real app, encrypt this. For now, plain text config.
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
        fs::write(&self.config_path, content)?;
        Ok(())
    }

    pub fn add(&mut self, config: DbConfig) {
        self.connections.push(config);
        let _ = self.save();
    }
    
    pub fn update(&mut self, config: DbConfig) {
        if let Some(pos) = self.connections.iter().position(|c| c.id == config.id) {
            self.connections[pos] = config;
            let _ = self.save();
        }
    }

    pub fn delete(&mut self, id: &str) {
        self.connections.retain(|c| c.id != id);
        let _ = self.save();
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryResult {
    pub columns: Vec<String>,
    pub rows: Vec<Vec<String>>, // Simple string display
    pub affected_rows: u64,
    pub execution_time_ms: u128,
}

#[async_trait::async_trait]
pub trait DatabaseExecutor {
    async fn execute(&self, config: &DbConfig, sql: &str) -> anyhow::Result<QueryResult>;
}

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
                    let val: String = match row.try_get_raw(i) {
                        Ok(value) => {
                            if value.is_null() {
                                "NULL".to_string()
                            } else {
                                format!("{:?}", value)
                            }
                        }
                        Err(_) => "ERR".to_string(),
                    };
                    row_data.push(val);
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

        let url = config.url.trim_start_matches("jdbc:sqlserver://");
        let parts: Vec<&str> = url.splitn(2, ';').collect();
        let addr_parts: Vec<&str> = parts[0].split(':').collect();

        let host = addr_parts[0];
        let port: u16 = if addr_parts.len() > 1 {
            addr_parts[1].parse().unwrap_or(1433)
        } else {
            1433
        };

        let mut t_config = Config::new();
        t_config.host(host);
        t_config.port(port);
        t_config.authentication(AuthMethod::sql_server(&config.user, &config.password));

        if parts.len() > 1 {
            for param in parts[1].split(';') {
                if param.is_empty() {
                    continue;
                }
                let kv: Vec<&str> = param.splitn(2, '=').collect();
                if kv.len() == 2 {
                    let key = kv[0].to_lowercase();
                    let val = kv[1];
                    match key.as_str() {
                        "databasename" => {
                            t_config.database(val);
                        }
                        "encrypt" => {
                            if val == "true" {
                                t_config.encryption(tiberius::EncryptionLevel::Required);
                            }
                        }
                        "trustservercertificate" => {
                            if val == "true" {
                                t_config.trust_cert();
                            }
                        }
                        _ => {}
                    }
                }
            }
        }

        let tcp = TcpStream::connect(t_config.get_addr()).await?;
        tcp.set_nodelay(true)?;

        let mut client = Client::connect(t_config, tcp.compat_write()).await?;

        let mut result = QueryResult {
            columns: Vec::new(),
            rows: Vec::new(),
            affected_rows: 0,
            execution_time_ms: 0,
        };

        let mut stream = client.query(sql, &[]).await?;

        if let Some(columns) = stream.columns().await? {
            result.columns = columns.iter().map(|c| c.name().to_string()).collect();
        }

        while let Some(item) = stream.next().await {
            match item? {
                tiberius::QueryItem::Row(row) => {
                    let mut row_data = Vec::new();
                    for i in 0..result.columns.len() {
                        let val: String = if let Ok(Some(s)) = row.try_get::<&str, _>(i) {
                            s.to_string()
                        } else if let Ok(Some(n)) = row.try_get::<i64, _>(i) {
                            n.to_string()
                        } else if let Ok(Some(n)) = row.try_get::<i32, _>(i) {
                            n.to_string()
                        } else if let Ok(Some(f)) = row.try_get::<f64, _>(i) {
                            f.to_string()
                        } else if let Ok(Some(b)) = row.try_get::<bool, _>(i) {
                            b.to_string()
                        } else {
                            "NULL/Unsupported".to_string()
                        };
                        row_data.push(val);
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

    pub async fn execute_query(&self, config: &DbConfig, sql: &str) -> anyhow::Result<QueryResult> {
        if config.db_type == DbType::SqlServer {
            return self.mssql_executor.execute(config, sql).await;
        }
        self.sqlx_executor.execute(config, sql).await
    }
}
