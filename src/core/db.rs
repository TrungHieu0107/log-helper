use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::future::Future;
use std::path::PathBuf;
use std::pin::Pin;
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
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

pub trait DatabaseExecutor: Send + Sync {
    fn execute_query<'a>(
        &'a self,
        config: &'a DbConfig,
        sql: &'a str,
    ) -> Pin<Box<dyn Future<Output = anyhow::Result<QueryResult>> + Send + 'a>>;
}

#[derive(Default)]
pub struct DbExecutorRegistry {
    executors: HashMap<DbType, Box<dyn DatabaseExecutor>>,
}

impl DbExecutorRegistry {
    pub fn new() -> Self {
        Self {
            executors: HashMap::new(),
        }
    }

    pub fn register<E>(&mut self, db_type: DbType, executor: E)
    where
        E: DatabaseExecutor + 'static,
    {
        self.executors.insert(db_type, Box::new(executor));
    }

    pub fn get(&self, db_type: &DbType) -> Option<&dyn DatabaseExecutor> {
        self.executors.get(db_type).map(|exec| exec.as_ref())
    }
}

#[derive(Clone)]
pub struct DbClient {
    registry: Arc<DbExecutorRegistry>,
}

impl DbClient {
    pub fn new(registry: Arc<DbExecutorRegistry>) -> Self {
        Self { registry }
    }

    pub async fn execute_query(&self, config: &DbConfig, sql: &str) -> anyhow::Result<QueryResult> {
        let executor = self.registry.get(&config.db_type).ok_or_else(|| {
            anyhow::anyhow!("No database executor registered for {}", config.db_type)
        })?;
        executor.execute_query(config, sql).await
    }
}

pub struct SqlxExecutor;

impl DatabaseExecutor for SqlxExecutor {
    fn execute_query<'a>(
        &'a self,
        config: &'a DbConfig,
        sql: &'a str,
    ) -> Pin<Box<dyn Future<Output = anyhow::Result<QueryResult>> + Send + 'a>> {
        Box::pin(async move {
            use sqlx::any::{AnyConnectOptions, AnyPoolOptions};
            use sqlx::{Column, Row, ValueRef};
            use std::str::FromStr;
            use std::time::Instant;

            let start = Instant::now();

            // Convert connection_url to sqlx options
            let opts = AnyConnectOptions::from_str(&config.url)?;

            let pool = AnyPoolOptions::new()
                .max_connections(1)
                .connect_with(opts)
                .await?;

            // Support multiple queries or just one? Usually one.
            // We'll use fetch_all for results and execute for non-SELECT.
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
                let rows = sqlx::query(sql).fetch_all(&pool).await?;

                if let Some(first_row) = rows.first() {
                    result.columns = first_row
                        .columns()
                        .iter()
                        .map(|c| c.name().to_string())
                        .collect();
                }

                for row in rows {
                    let mut row_data = Vec::new();
                    for i in 0..result.columns.len() {
                        let val: String = match row.try_get_raw(i) {
                            Ok(value) => {
                                if value.is_null() {
                                    "NULL".to_string()
                                } else {
                                    format!("{:?}", value) // Fallback to debug
                                }
                            }
                            Err(_) => "ERR".to_string(),
                        };
                        row_data.push(val);
                    }
                    result.rows.push(row_data);
                }
            } else {
                let res = sqlx::query(sql).execute(&pool).await?;
                result.affected_rows = res.rows_affected();
            }

            result.execution_time_ms = start.elapsed().as_millis();
            Ok(result)
        })
    }
}

pub struct MsSqlExecutor;

impl DatabaseExecutor for MsSqlExecutor {
    fn execute_query<'a>(
        &'a self,
        config: &'a DbConfig,
        sql: &'a str,
    ) -> Pin<Box<dyn Future<Output = anyhow::Result<QueryResult>> + Send + 'a>> {
        Box::pin(async move {
            use futures_util::stream::StreamExt;
            use std::time::Instant;
            use tiberius::{AuthMethod, Client, Config};
            use tokio::net::TcpStream;
            use tokio_util::compat::TokioAsyncWriteCompatExt;

            let start = Instant::now();

            // Parse JDBC URL: jdbc:sqlserver://host:port;databaseName=DBNAME;encrypt=true;trustServerCertificate=true
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

            // Get columns from the first result set
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
                                // Fallback for NULL or unhandled types (like dates)
                                "NULL/Unsupported".to_string()
                            };
                            row_data.push(val);
                        }
                        result.rows.push(row_data);
                    }
                    tiberius::QueryItem::Metadata(_) => {}
                }
            }

            // Drop the stream explicitly to release mutable borrow on client
            drop(stream);

            // To get affected rows for UPDATE/INSERT, we might need to use client.execute
            // but let's try to get it from the stream if possible, or use a separate branch
            if result.rows.is_empty() && result.columns.is_empty() {
                // Re-run as execute if no rows found and it's likely a DML
                let counts = client.execute(sql, &[]).await?;
                result.affected_rows = counts.total();
            }

            result.execution_time_ms = start.elapsed().as_millis();
            Ok(result)
        })
    }
}
