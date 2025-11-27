# Multi-Database Architecture Sketch

## Current vs Proposed Architecture

### Current (v0.5.4)
```
DatabaseManager (Concrete Struct)
├── rusqlite::Connection (hardcoded)
├── connect_database() → SQLCipher specific
├── get_tables() → sqlite_master queries
└── compare_schemas() → works with rusqlite
```

### Proposed (v2.0 - Multi-DB Support)
```
DatabaseProvider (Trait) ← Define common interface
├── SQLiteProvider (Impl)
│   └── rusqlite::Connection
└── MySQLProvider (Impl)
    └── mysql_async::Conn

DatabaseManager (Generic)
├── Box<dyn DatabaseProvider> ← Can hold ANY database
└── compare_schemas() ← Works with trait, DB-agnostic
```

---

## 1. Core Trait Definition

```rust
// src-tauri/src/database/provider.rs

use async_trait::async_trait;
use crate::models::*;
use anyhow::Result;

/// Common interface for all database types
#[async_trait]
pub trait DatabaseProvider: Send + Sync {
    /// Connect to database with connection info
    async fn connect(&mut self, connection_info: &ConnectionInfo) -> Result<DatabaseInfo>;

    /// Disconnect and cleanup
    async fn disconnect(&mut self) -> Result<()>;

    /// Get list of tables in the database
    async fn get_tables(&self) -> Result<Vec<TableInfo>>;

    /// Get columns for a specific table
    async fn get_table_columns(&self, table_name: &str) -> Result<Vec<ColumnInfo>>;

    /// Get table data with pagination
    async fn get_table_data(
        &self,
        table_name: &str,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> Result<TableData>;

    /// Get database metadata
    fn get_database_info(&self) -> &DatabaseInfo;

    /// Check if connected
    fn is_connected(&self) -> bool;
}
```

---

## 2. Connection Info (Database-Agnostic)

```rust
// src-tauri/src/models.rs

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum ConnectionInfo {
    // File-based databases (SQLite, SQLCipher)
    FileBased {
        path: String,
        password: Option<String>,
        settings: Option<serde_json::Value>,
    },

    // Network-based databases (MySQL, PostgreSQL)
    NetworkBased {
        host: String,
        port: u16,
        username: String,
        password: String,
        database: String,
        ssl: bool,
    },
}

#[derive(Debug, Clone, serde::Serialize)]
pub enum DatabaseType {
    SQLite,
    SQLCipher,
    MySQL,
    PostgreSQL,  // Future
}
```

---

## 3. SQLite Provider (Current Code Wrapped)

```rust
// src-tauri/src/database/sqlite_provider.rs

use super::provider::DatabaseProvider;
use rusqlite::Connection;
use async_trait::async_trait;

pub struct SQLiteProvider {
    connection: Option<Connection>,
    db_info: Option<DatabaseInfo>,
}

impl SQLiteProvider {
    pub fn new() -> Self {
        Self {
            connection: None,
            db_info: None,
        }
    }
}

#[async_trait]
impl DatabaseProvider for SQLiteProvider {
    async fn connect(&mut self, info: &ConnectionInfo) -> Result<DatabaseInfo> {
        match info {
            ConnectionInfo::FileBased { path, password, settings } => {
                // YOUR EXISTING LOGIC from database.rs:163-289
                let conn = Connection::open(path)?;

                // If password provided, it's SQLCipher
                if let Some(pwd) = password {
                    conn.pragma_update(None, "key", pwd)?;
                    // Apply settings...
                }

                // Get table count
                let table_count: i32 = conn.query_row(
                    "SELECT COUNT(*) FROM sqlite_master WHERE type='table';",
                    [],
                    |row| row.get(0),
                )?;

                let db_info = DatabaseInfo {
                    path: path.clone(),
                    name: Path::new(path).file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_string(),
                    table_count,
                    is_connected: true,
                    alias: None,
                    password: password.clone(),
                };

                self.connection = Some(conn);
                self.db_info = Some(db_info.clone());
                Ok(db_info)
            }
            _ => Err(anyhow::anyhow!("SQLite only supports file-based connections")),
        }
    }

    async fn get_tables(&self) -> Result<Vec<TableInfo>> {
        let conn = self.connection.as_ref()
            .ok_or(anyhow::anyhow!("Not connected"))?;

        // YOUR EXISTING LOGIC from database.rs:291-328
        let mut stmt = conn.prepare(
            "SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%' ORDER BY name"
        )?;

        // ... rest of your code
        Ok(Vec::new()) // placeholder
    }

    async fn get_table_columns(&self, table_name: &str) -> Result<Vec<ColumnInfo>> {
        let conn = self.connection.as_ref()
            .ok_or(anyhow::anyhow!("Not connected"))?;

        // YOUR EXISTING LOGIC from database.rs:330-350
        let mut stmt = conn.prepare(&format!("PRAGMA table_info(\"{}\")", table_name))?;

        // ... rest of your code
        Ok(Vec::new()) // placeholder
    }

    async fn get_table_data(
        &self,
        table_name: &str,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> Result<TableData> {
        // YOUR EXISTING LOGIC from database.rs:411-483
        Ok(TableData {
            columns: Vec::new(),
            rows: Vec::new(),
            total_count: 0,
        })
    }

    fn get_database_info(&self) -> &DatabaseInfo {
        self.db_info.as_ref().unwrap()
    }

    fn is_connected(&self) -> bool {
        self.connection.is_some()
    }

    async fn disconnect(&mut self) -> Result<()> {
        self.connection = None;
        Ok(())
    }
}
```

---

## 4. MySQL Provider (New Implementation)

```rust
// src-tauri/src/database/mysql_provider.rs

use super::provider::DatabaseProvider;
use mysql_async::{Conn, Pool, Opts, OptsBuilder};
use async_trait::async_trait;

pub struct MySQLProvider {
    pool: Option<Pool>,
    connection: Option<Conn>,
    db_info: Option<DatabaseInfo>,
}

impl MySQLProvider {
    pub fn new() -> Self {
        Self {
            pool: None,
            connection: None,
            db_info: None,
        }
    }
}

#[async_trait]
impl DatabaseProvider for MySQLProvider {
    async fn connect(&mut self, info: &ConnectionInfo) -> Result<DatabaseInfo> {
        match info {
            ConnectionInfo::NetworkBased { host, port, username, password, database, ssl } => {
                // Build MySQL connection
                let opts = OptsBuilder::default()
                    .ip_or_hostname(host)
                    .tcp_port(*port)
                    .user(Some(username))
                    .pass(Some(password))
                    .db_name(Some(database));

                let pool = Pool::new(opts);
                let mut conn = pool.get_conn().await?;

                // Get table count
                let table_count: i32 = conn.query_first(
                    format!("SELECT COUNT(*) FROM information_schema.tables WHERE table_schema = '{}'", database)
                ).await?.unwrap_or(0);

                let db_info = DatabaseInfo {
                    path: format!("{}:{}/{}", host, port, database),
                    name: database.clone(),
                    table_count,
                    is_connected: true,
                    alias: None,
                    password: Some(password.clone()),
                };

                self.pool = Some(pool);
                self.connection = Some(conn);
                self.db_info = Some(db_info.clone());
                Ok(db_info)
            }
            _ => Err(anyhow::anyhow!("MySQL only supports network-based connections")),
        }
    }

    async fn get_tables(&self) -> Result<Vec<TableInfo>> {
        let conn = self.connection.as_ref()
            .ok_or(anyhow::anyhow!("Not connected"))?;

        let db_name = &self.db_info.as_ref().unwrap().name;

        // MySQL equivalent of sqlite_master
        let query = format!(
            "SELECT table_name FROM information_schema.tables
             WHERE table_schema = '{}' AND table_type = 'BASE TABLE'
             ORDER BY table_name",
            db_name
        );

        let table_names: Vec<String> = conn.query(query).await?;

        let mut tables = Vec::new();
        for table_name in table_names {
            let row_count: i64 = conn.query_first(
                format!("SELECT COUNT(*) FROM `{}`", table_name)
            ).await?.unwrap_or(0);

            let columns = self.get_table_columns(&table_name).await?;

            tables.push(TableInfo {
                name: table_name,
                row_count,
                columns,
            });
        }

        Ok(tables)
    }

    async fn get_table_columns(&self, table_name: &str) -> Result<Vec<ColumnInfo>> {
        let conn = self.connection.as_ref()
            .ok_or(anyhow::anyhow!("Not connected"))?;

        let db_name = &self.db_info.as_ref().unwrap().name;

        // MySQL equivalent of PRAGMA table_info
        let query = format!(
            "SELECT
                column_name,
                data_type,
                is_nullable,
                column_default,
                column_key
             FROM information_schema.columns
             WHERE table_schema = '{}' AND table_name = '{}'
             ORDER BY ordinal_position",
            db_name, table_name
        );

        let rows: Vec<(String, String, String, Option<String>, String)> =
            conn.query(query).await?;

        let columns = rows.into_iter().map(|(name, data_type, nullable, default_val, key)| {
            ColumnInfo {
                name,
                data_type,
                is_nullable: nullable == "YES",
                default_value: default_val,
                is_primary_key: key == "PRI",
            }
        }).collect();

        Ok(columns)
    }

    async fn get_table_data(
        &self,
        table_name: &str,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> Result<TableData> {
        let conn = self.connection.as_ref()
            .ok_or(anyhow::anyhow!("Not connected"))?;

        // Similar to SQLite but with MySQL syntax
        let columns = self.get_table_columns(table_name).await?;
        let column_names: Vec<String> = columns.iter().map(|c| c.name.clone()).collect();

        let total_count: i64 = conn.query_first(
            format!("SELECT COUNT(*) FROM `{}`", table_name)
        ).await?.unwrap_or(0);

        let col_list = column_names.iter()
            .map(|c| format!("`{}`", c))
            .join(", ");

        let query = match (limit, offset) {
            (Some(l), Some(o)) => format!("SELECT {} FROM `{}` LIMIT {} OFFSET {}", col_list, table_name, l, o),
            (Some(l), None) => format!("SELECT {} FROM `{}` LIMIT {}", col_list, table_name, l),
            (None, Some(o)) => format!("SELECT {} FROM `{}` OFFSET {}", col_list, table_name, o),
            (None, None) => format!("SELECT {} FROM `{}`", col_list, table_name),
        };

        // Execute and convert to Vec<Vec<serde_json::Value>>
        let rows: Vec<Vec<serde_json::Value>> = conn.query(query).await?;

        Ok(TableData {
            columns: column_names,
            rows,
            total_count,
        })
    }

    fn get_database_info(&self) -> &DatabaseInfo {
        self.db_info.as_ref().unwrap()
    }

    fn is_connected(&self) -> bool {
        self.connection.is_some()
    }

    async fn disconnect(&mut self) -> Result<()> {
        self.connection = None;
        if let Some(pool) = self.pool.take() {
            pool.disconnect().await?;
        }
        Ok(())
    }
}
```

---

## 5. Updated DatabaseManager (Generic)

```rust
// src-tauri/src/database/manager.rs

use std::collections::HashMap;
use crate::models::*;

pub struct DatabaseManager {
    // Store different database types by ID
    providers: HashMap<String, Box<dyn DatabaseProvider>>,
}

impl DatabaseManager {
    pub fn new() -> Self {
        Self {
            providers: HashMap::new(),
        }
    }

    /// Connect to any database (auto-detect type from ConnectionInfo)
    pub async fn connect_database(
        &mut self,
        id: String,
        connection_info: &ConnectionInfo,
    ) -> Result<DatabaseInfo> {
        // Choose provider based on connection type
        let mut provider: Box<dyn DatabaseProvider> = match connection_info {
            ConnectionInfo::FileBased { password, .. } => {
                if password.is_some() {
                    Box::new(SQLiteProvider::new()) // SQLCipher
                } else {
                    Box::new(SQLiteProvider::new()) // Plain SQLite
                }
            }
            ConnectionInfo::NetworkBased { .. } => {
                Box::new(MySQLProvider::new()) // MySQL
            }
        };

        let db_info = provider.connect(connection_info).await?;
        self.providers.insert(id, provider);
        Ok(db_info)
    }

    /// Compare schemas - WORKS WITH ANY DATABASE TYPE
    pub async fn compare_schemas(
        &self,
        db1_id: &str,
        db2_id: &str,
    ) -> Result<SchemaComparison> {
        let provider1 = self.providers.get(db1_id)
            .ok_or(anyhow::anyhow!("Database 1 not connected"))?;
        let provider2 = self.providers.get(db2_id)
            .ok_or(anyhow::anyhow!("Database 2 not connected"))?;

        // Get tables from BOTH databases (doesn't matter which type!)
        let tables1 = provider1.get_tables().await?;
        let tables2 = provider2.get_tables().await?;

        // YOUR EXISTING COMPARISON LOGIC - UNCHANGED!
        let table1_names: HashSet<String> =
            tables1.iter().map(|t| t.name.clone()).collect();
        let table2_names: HashSet<String> =
            tables2.iter().map(|t| t.name.clone()).collect();

        let added_tables: Vec<String> =
            table2_names.difference(&table1_names).cloned().collect();
        let removed_tables: Vec<String> =
            table1_names.difference(&table2_names).cloned().collect();

        // ... rest of your existing comparison logic

        Ok(SchemaComparison {
            database1: provider1.get_database_info().path.clone(),
            database2: provider2.get_database_info().path.clone(),
            added_tables,
            removed_tables,
            modified_tables: Vec::new(),
            identical_tables: Vec::new(),
        })
    }
}
```

---

## 6. Tauri Commands (Minimal Changes)

```rust
// src-tauri/src/commands.rs

#[tauri::command]
pub async fn connect_database(
    db_manager: State<'_, Mutex<DatabaseManager>>,
    connection_info: ConnectionInfo,  // ← Now accepts both file & network
) -> Result<DatabaseInfo, String> {
    let mut manager = db_manager.lock().unwrap();

    // Generate unique ID
    let id = match &connection_info {
        ConnectionInfo::FileBased { path, .. } => path.clone(),
        ConnectionInfo::NetworkBased { host, database, .. } =>
            format!("{}:{}", host, database),
    };

    manager.connect_database(id, &connection_info)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn compare_schemas(
    db_manager: State<'_, Mutex<DatabaseManager>>,
    db1_id: String,
    db2_id: String,
) -> Result<SchemaComparison, String> {
    let manager = db_manager.lock().unwrap();

    manager.compare_schemas(&db1_id, &db2_id)
        .await
        .map_err(|e| e.to_string())
}
```

---

## 7. Frontend Changes (Minimal)

```vue
<!-- src/components/DatabaseConnect.vue -->

<script setup lang="ts">
// Add database type selector
const dbType = ref<'file' | 'network'>('file');

// File-based connection
const fileConnection = {
  path: '/path/to/database.db',
  password: 'optional',
  settings: {}
};

// Network-based connection
const networkConnection = {
  host: 'localhost',
  port: 3306,
  username: 'root',
  password: 'secret',
  database: 'mydb',
  ssl: false
};

async function connect() {
  const connectionInfo = dbType.value === 'file'
    ? { FileBased: fileConnection }
    : { NetworkBased: networkConnection };

  await invoke('connect_database', { connectionInfo });
}
</script>

<template>
  <!-- Radio buttons to switch between File/Network -->
  <div>
    <input type="radio" v-model="dbType" value="file"> SQLite/SQLCipher
    <input type="radio" v-model="dbType" value="network"> MySQL
  </div>

  <!-- Show different forms based on dbType -->
  <div v-if="dbType === 'file'">
    <input v-model="fileConnection.path" placeholder="Database path">
    <input v-model="fileConnection.password" placeholder="Password (optional)">
  </div>

  <div v-else>
    <input v-model="networkConnection.host" placeholder="Host">
    <input v-model="networkConnection.port" placeholder="Port">
    <input v-model="networkConnection.username" placeholder="Username">
    <input v-model="networkConnection.password" type="password" placeholder="Password">
    <input v-model="networkConnection.database" placeholder="Database name">
  </div>
</template>
```

---

## 8. Cargo.toml Dependencies

```toml
[dependencies]
# Existing
rusqlite = { version = "0.32", features = ["bundled", "sqlcipher", "bundled-sqlcipher"] }

# New (add these for MySQL support)
mysql_async = "0.34"
async-trait = "0.1"

# Already have tokio, so async support is ready
tokio = { version = "1.0", features = ["full", "time"] }
```

---

## Migration Path

### Phase 1: Launch Beta (Now) ✅
- Keep current architecture
- Ship SQLCipher-only version
- Get user feedback

### Phase 2: CLI (Week 1-2) 🚀
- Add `clap` for CLI args
- Call existing functions
- Easy win, minimal code

### Phase 3: Refactor to Traits (Week 3-4) 🔧
- Create `DatabaseProvider` trait
- Wrap existing code in `SQLiteProvider`
- No functionality change, just architecture prep
- **Test thoroughly** - should behave identically

### Phase 4: Add MySQL (Week 5-8) 🆕
- Implement `MySQLProvider`
- Add connection UI toggle
- Test cross-database comparisons (SQLite vs MySQL!)

---

## Benefits of This Architecture

1. **Your comparison logic NEVER changes** - it works with the trait
2. **Easy to add PostgreSQL, MongoDB, etc.** - just implement the trait
3. **Can compare SQLite vs MySQL** - both implement same interface
4. **Frontend mostly unchanged** - just pass different connection info
5. **Testable** - can mock `DatabaseProvider` for unit tests

---

## Complexity Estimate

| Task | Lines of Code | Effort |
|------|---------------|--------|
| Define traits | ~100 | 2 hours |
| Wrap existing SQLite code | ~200 | 4 hours |
| Refactor DatabaseManager | ~150 | 3 hours |
| Implement MySQL provider | ~400 | 2-3 days |
| Update frontend UI | ~100 | 1 day |
| Testing | - | 2-3 days |
| **TOTAL** | **~950 lines** | **1-2 weeks** |

Compare to CLI: ~50 lines, 1-2 days.

**Recommendation**: Do CLI first, then refactor to traits when you have time!
