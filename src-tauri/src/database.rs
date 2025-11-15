
use rusqlite::{Connection, Result as RusqliteResult, types::Value};
use crate::models::*;
use std::collections::HashMap;
// use anyhow::{Context, Result};  
use anyhow::{Context};
use std::path::Path;

pub struct DatabaseManager {
    connections: HashMap<String, Connection>,
    connected_databases: HashMap<String, DatabaseInfo>, 
}


#[derive(Debug)]
pub enum DatabaseType {
    SQLite,      // Regular unencrypted SQLite
    SQLCipher,   // Encrypted SQLCipher
}

impl DatabaseManager {



    
    pub fn new() -> Self {
        Self {
            connections: HashMap::new(),
            connected_databases: HashMap::new(),
        }
    }

    pub fn get_password(&self, db_path: &str) -> Option<String> {
        self.connected_databases.get(db_path).and_then(|info| info.password.clone())
    }

    /// Detect if a database is encrypted (SQLCipher) or regular SQLite
    fn detect_database_type(&self, path: &str) -> anyhow::Result<DatabaseType> {
        // Use Path for cross-platform handling
        let db_path = Path::new(path);

        // Try to open as regular SQLite first
        match Connection::open(db_path) {
            Ok(conn) => {
                // Try to read sqlite_master without password
                match conn.query_row("SELECT COUNT(*) FROM sqlite_master;", [], |row| row.get::<_, i32>(0)) {
                    Ok(_) => {
                        println!("Database detected as: Regular SQLite (unencrypted)");
                        Ok(DatabaseType::SQLite)
                    },
                    Err(_) => {
                        // If we can't read, it might be encrypted
                        println!("Database detected as: SQLCipher (encrypted)");
                        Ok(DatabaseType::SQLCipher)
                    }
                }
            },
            Err(e) => {
                Err(anyhow::anyhow!("Failed to open database file: {}", e))
            }
        }
    }



    //     pub fn connect_database(&mut self, path: &str, password: &str) -> anyhow::Result<DatabaseInfo> {
    //     println!("Attempting to connect to database: {}", path);
        
    //     let db_path = Path::new(path);
    //     let db_type = self.detect_database_type(path)?;
        
    //     match db_type {
    //         DatabaseType::SQLite => {
    //             let conn = Connection::open(db_path)
    //                 .with_context(|| format!("Failed to open database file: {}", path))?;
                
    //             let table_count: i32 = conn.query_row(
    //                 "SELECT COUNT(*) FROM sqlite_master WHERE type='table';", 
    //                 [], 
    //                 |row| row.get(0)
    //             )?;
                
    //             let db_info = DatabaseInfo {
    //                 path: path.to_string(),
    //                 name: db_path.file_name().unwrap_or_default().to_string_lossy().to_string(),
    //                 table_count,
    //                 is_connected: true,
    //                 alias: None,
    //                 password: Some(password.to_string()), // Store the password
    //             };
                
    //             self.connections.insert(path.to_string(), conn);
    //             self.connected_databases.insert(path.to_string(), db_info.clone()); // Store in both maps
    //             Ok(db_info)
    //         }
    //         DatabaseType::SQLCipher => {
    //             let conn = Connection::open(db_path)
    //                 .with_context(|| format!("Failed to open database file: {}", path))?;
                
    //             let key_formats = vec![
    //                 format!("PRAGMA key = '{}';", password),
    //                 format!("PRAGMA key = \"{}\";", password),
    //                 format!("PRAGMA key = {};", password),
    //                 format!("PRAGMA key='{}';", password),
    //             ];

    //             let mut sqlcipher_success = false;
    //             let mut last_error = String::new();
    //             let mut table_count = 0;

    //             for key_format in key_formats {
    //                 println!("Trying SQLCipher key format: {}", key_format.replace(password, "****"));
                    
    //                 if let Err(e) = conn.execute_batch(&key_format) {
    //                     last_error = format!("Key format failed: {}", e);
    //                     continue;
    //                 }
                    
    //                 match conn.query_row("SELECT COUNT(*) FROM sqlite_master WHERE type='table';", [], |row| row.get::<_, i32>(0)) {
    //                     Ok(count) => {
    //                         println!("SQLCipher key accepted! Found {} tables.", count);
    //                         sqlcipher_success = true;
    //                         table_count = count;
    //                         break;
    //                     },
    //                     Err(e) => {
    //                         last_error = format!("Key verification failed: {}", e);
    //                         continue;
    //                     }
    //                 }
    //             }

    //             if !sqlcipher_success {
    //                 return Err(anyhow::anyhow!("Failed to connect to SQLCipher database. Last error: {}", last_error));
    //             }

    //             let db_name = db_path
    //                 .file_name()
    //                 .unwrap_or_default()
    //                 .to_string_lossy()
    //                 .to_string();

    //             let db_info = DatabaseInfo {
    //                 path: path.to_string(),
    //                 name: db_name,
    //                 table_count,
    //                 is_connected: true,
    //                 alias: None,
    //                 password: Some(password.to_string()), // Store the password
    //             };

    //             self.connections.insert(path.to_string(), conn);
    //             self.connected_databases.insert(path.to_string(), db_info.clone());
    //             println!("Database connection stored successfully");
                
    //             Ok(db_info)
    //         }
    //     }
    // }

    /// Connect to database with automatic type detection
    /// 
pub fn connect_database(
    &mut self, 
    path: &str, 
    password: &str,
    settings: Option<serde_json::Value> // Add this parameter
) -> anyhow::Result<DatabaseInfo> {
    println!("Attempting to connect to database: {}", path);
    
    let db_path = Path::new(path);
    let db_type = self.detect_database_type(path)?;
    
    match db_type {
        DatabaseType::SQLite => {
            // SQLite connection (no password/settings needed)
            let conn = Connection::open(db_path)
                .with_context(|| format!("Failed to open database file: {}", path))?;
            
            let table_count: i32 = conn.query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table';", 
                [], 
                |row| row.get(0)
            )?;
            
            let db_info = DatabaseInfo {
                path: path.to_string(),
                name: db_path.file_name().unwrap_or_default().to_string_lossy().to_string(),
                table_count,
                is_connected: true,
                alias: None,
                password: Some(password.to_string()),
            };
            
            self.connections.insert(path.to_string(), conn);
            self.connected_databases.insert(path.to_string(), db_info.clone());
            Ok(db_info)
        }
        DatabaseType::SQLCipher => {
            let conn = Connection::open(db_path)
                .with_context(|| format!("Failed to open database file: {}", path))?;
            
            // Set password
            conn.pragma_update(None, "key", password)
                .with_context(|| "Failed to set encryption key")?;
            
            // Apply settings from user or use defaults
            if let Some(s) = settings {
                // Extract settings from JSON
                let page_size = s["page_size"].as_str().unwrap_or("4096");
                let kdf_iter = s["kdf_iterations"].as_str().unwrap_or("256000");
                let hmac_algo = s["hmac_algorithm"].as_str().unwrap_or("HMAC_SHA256");
                let kdf_algo = s["kdf_algorithm"].as_str().unwrap_or("PBKDF2_HMAC_SHA256");
                
                println!("📝 Applying user settings: page_size={}, kdf_iter={}", page_size, kdf_iter);
                
                conn.pragma_update(None, "cipher_page_size", page_size)
                    .with_context(|| "Failed to set page size")?;
                
                conn.pragma_update(None, "kdf_iter", kdf_iter)
                    .with_context(|| "Failed to set KDF iterations")?;
                
                conn.pragma_update(None, "cipher_hmac_algorithm", hmac_algo)
                    .with_context(|| "Failed to set HMAC algorithm")?;
                
                conn.pragma_update(None, "cipher_kdf_algorithm", kdf_algo)
                    .with_context(|| "Failed to set KDF algorithm")?;
            } else {
                // Use default settings (same as migration defaults)
                println!("📝 Applying default settings");
                
                conn.pragma_update(None, "cipher_page_size", "4096")
                    .with_context(|| "Failed to set page size")?;
                
                conn.pragma_update(None, "kdf_iter", "256000")
                    .with_context(|| "Failed to set KDF iterations")?;
                
                conn.pragma_update(None, "cipher_hmac_algorithm", "HMAC_SHA256")
                    .with_context(|| "Failed to set HMAC algorithm")?;
                
                conn.pragma_update(None, "cipher_kdf_algorithm", "PBKDF2_HMAC_SHA256")
                    .with_context(|| "Failed to set KDF algorithm")?;
            }
            
            println!("✅ SQLCipher settings applied");
            
            // Verify connection
            let table_count: i32 = conn.query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table';",
                [],
                |row| row.get(0)
            ).with_context(|| "Key verification failed - incorrect password or settings")?;
            
            println!("✅ SQLCipher key accepted! Found {} tables.", table_count);

            let db_name = db_path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();

            let db_info = DatabaseInfo {
                path: path.to_string(),
                name: db_name,
                table_count,
                is_connected: true,
                alias: None,
                password: Some(password.to_string()),
            };

            self.connections.insert(path.to_string(), conn);
            self.connected_databases.insert(path.to_string(), db_info.clone());
            println!("✅ Database connection stored successfully");
            
            Ok(db_info)
        }
    }
}


    pub fn get_tables(&self, db_path: &str) -> anyhow::Result<Vec<TableInfo>> {
        let conn = self.connections.get(db_path)
            .context("Database not connected")?;

        let mut stmt = conn.prepare(
            "SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%' ORDER BY name"
        )?;

        let table_names: Vec<String> = stmt.query_map([], |row| {
            Ok(row.get::<_, String>(0)?)
        })?.collect::<RusqliteResult<Vec<_>>>()?;

        let mut tables = Vec::new();
        
        for table_name in table_names {
            // Get row count safely
            let row_count: i64 = conn.query_row(
                &format!("SELECT COUNT(*) FROM \"{}\"", table_name),
                [],
                |row| row.get(0),
            ).unwrap_or(0);

            // Get column info
            let columns = self.get_table_columns(conn, &table_name)?;

            tables.push(TableInfo {
                name: table_name,
                row_count,
                columns,
            });
        }

        Ok(tables)
    }

    fn get_table_columns(&self, conn: &Connection, table_name: &str) -> anyhow::Result<Vec<ColumnInfo>> {
        let mut stmt = conn.prepare(&format!("PRAGMA table_info(\"{}\")", table_name))?;
        
        let columns: Vec<ColumnInfo> = stmt.query_map([], |row| {
            Ok(ColumnInfo {
                name: row.get::<_, String>(1)?,
                data_type: row.get::<_, String>(2)?,
                is_nullable: row.get::<_, i32>(3)? == 0,
                default_value: row.get::<_, Option<String>>(4)?,
                is_primary_key: row.get::<_, i32>(5)? == 1,
            })
        })?.collect::<RusqliteResult<Vec<_>>>()?;

        Ok(columns)
    }

    // pub fn get_table_data(&self, db_path: &str, table_name: &str, limit: Option<i64>) -> anyhow::Result<TableData> {
    //     let conn = self.connections.get(db_path)
    //         .context("Database not connected")?;

    //     // Get column names
    //     let columns = self.get_table_columns(conn, table_name)?;
    //     let column_names: Vec<String> = columns.iter().map(|c| c.name.clone()).collect();

    //     // Get total count
    //     let total_count: i64 = conn.query_row(
    //         &format!("SELECT COUNT(*) FROM \"{}\"", table_name),
    //         [],
    //         |row| row.get(0),
    //     )?;

    //     // Build dynamic SELECT query using column names to ensure order
    //     let col_list = column_names.iter()
    //         .map(|c| format!("\"{}\"", c))
    //         .collect::<Vec<_>>()
    //         .join(", ");
    //     let query = match limit {
    //         Some(limit) => format!("SELECT {} FROM \"{}\" LIMIT {}", col_list, table_name, limit),
    //         None => format!("SELECT {} FROM \"{}\"", col_list, table_name),
    //     };

    //     let mut stmt = conn.prepare(&query)?;
    //     let column_count = stmt.column_count();
        
    //     // Return rows as arrays (your existing format)
    //     let rows: Vec<Vec<serde_json::Value>> = stmt.query_map([], |row| {
    //         let mut row_data = Vec::new();
    //         for i in 0..column_count {
    //             let value: Value = row.get(i)?;
    //             let json_value = match value {
    //                 Value::Null => serde_json::Value::Null,
    //                 Value::Integer(i) => serde_json::Value::Number(i.into()),
    //                 Value::Real(f) => serde_json::Value::Number(
    //                     serde_json::Number::from_f64(f).unwrap_or(0.into())
    //                 ),
    //                 Value::Text(s) => serde_json::Value::String(s),
    //                 Value::Blob(b) => serde_json::Value::String(
    //                     format!("<BLOB {} bytes>", b.len())
    //                 ),
    //             };
    //             row_data.push(json_value);
    //         }
    //         Ok(row_data)
    //     })?.collect::<RusqliteResult<Vec<_>>>()?;

    //     Ok(TableData {
    //         columns: column_names,
    //         rows,
    //         total_count,
    //     })
    // }



    // REPLACE the existing get_table_data function in database.rs (around line 303)
// with this version that supports OFFSET



pub fn get_table_data(
    &self, 
    db_path: &str, 
    table_name: &str, 
    limit: Option<i64>,
    offset: Option<i64>  // ← NEW PARAMETER
) -> anyhow::Result<TableData> {
    let conn = self.connections.get(db_path)
        .context("Database not connected")?;

    // Get column names
    let columns = self.get_table_columns(conn, table_name)?;
    let column_names: Vec<String> = columns.iter().map(|c| c.name.clone()).collect();

    // Get total count
    let total_count: i64 = conn.query_row(
        &format!("SELECT COUNT(*) FROM \"{}\"", table_name),
        [],
        |row| row.get(0),
    )?;

    // Build dynamic SELECT query using column names to ensure order
    let col_list = column_names.iter()
        .map(|c| format!("\"{}\"", c))
        .collect::<Vec<_>>()
        .join(", ");
    
    // ← UPDATED: Build query with OFFSET support
    let query = match (limit, offset) {
        (Some(l), Some(o)) => format!(
            "SELECT {} FROM \"{}\" LIMIT {} OFFSET {}", 
            col_list, table_name, l, o
        ),
        (Some(l), None) => format!(
            "SELECT {} FROM \"{}\" LIMIT {}", 
            col_list, table_name, l
        ),
        (None, Some(o)) => format!(
            "SELECT {} FROM \"{}\" OFFSET {}", 
            col_list, table_name, o
        ),
        (None, None) => format!(
            "SELECT {} FROM \"{}\"", 
            col_list, table_name
        ),
    };

    let mut stmt = conn.prepare(&query)?;
    let column_count = stmt.column_count();
    
    // Return rows as arrays (your existing format)
    let rows: Vec<Vec<serde_json::Value>> = stmt.query_map([], |row| {
        let mut row_data = Vec::new();
        for i in 0..column_count {
            let value: Value = row.get(i)?;
            let json_value = match value {
                Value::Null => serde_json::Value::Null,
                Value::Integer(i) => serde_json::Value::Number(i.into()),
                Value::Real(f) => serde_json::Value::Number(
                    serde_json::Number::from_f64(f).unwrap_or(0.into())
                ),
                Value::Text(s) => serde_json::Value::String(s),
                Value::Blob(b) => serde_json::Value::String(
                    format!("<BLOB {} bytes>", b.len())
                ),
            };
            row_data.push(json_value);
        }
        Ok(row_data)
    })?.collect::<RusqliteResult<Vec<_>>>()?;

    Ok(TableData {
        columns: column_names,
        rows,
        total_count,
    })
}
    pub fn compare_schemas(&self, db1_path: &str, db2_path: &str) -> anyhow::Result<SchemaComparison> {
        let tables1 = self.get_tables(db1_path)?;
        let tables2 = self.get_tables(db2_path)?;

        let table1_names: std::collections::HashSet<String> = tables1.iter().map(|t| t.name.clone()).collect();
        let table2_names: std::collections::HashSet<String> = tables2.iter().map(|t| t.name.clone()).collect();

        let added_tables: Vec<String> = table2_names.difference(&table1_names).cloned().collect();
        let removed_tables: Vec<String> = table1_names.difference(&table2_names).cloned().collect();
        
        let common_tables: Vec<String> = table1_names.intersection(&table2_names).cloned().collect();
        let mut modified_tables = Vec::new();
        let mut identical_tables = Vec::new();

        for table_name in common_tables {
            let table1 = tables1.iter().find(|t| t.name == table_name).unwrap();
            let table2 = tables2.iter().find(|t| t.name == table_name).unwrap();
            
            if self.tables_are_identical(&table1, &table2) {
                identical_tables.push(table_name);
            } else {
                let diff = self.compare_tables(&table1, &table2);
                modified_tables.push(diff);
            }
        }

        Ok(SchemaComparison {
            database1: db1_path.to_string(),
            database2: db2_path.to_string(),
            added_tables,
            removed_tables,
            modified_tables,
            identical_tables,
        })
    }

// Fast data comparison using SQL joins - handles 500K+ rows easily
pub fn compare_table_data_fast(
    &self,
    db1_path: &str,
    db2_path: &str,
    table_name: &str,
    primary_key: &str,
) -> anyhow::Result<DataComparisonResult> {
    let conn1 = self.connections.get(db1_path)
        .context("Database 1 not connected")?;
    let _conn2 = self.connections.get(db2_path)
        .context("Database 2 not connected")?;
    
    // Attach second database to first connection
    conn1.execute(
        &format!("ATTACH DATABASE '{}' AS db2", db2_path),
        [],
    )?;
    
    // Get total row counts (fast)
    let count1: i64 = conn1.query_row(
        &format!("SELECT COUNT(*) FROM {}", table_name),
        [],
        |row| row.get(0)
    )?;
    
    let count2: i64 = conn1.query_row(
        &format!("SELECT COUNT(*) FROM db2.{}", table_name),
        [],
        |row| row.get(0)
    )?;
    
    // Find rows only in DB1 (deleted rows)
    let deleted_count: i64 = conn1.query_row(
        &format!(
            "SELECT COUNT(*) FROM {} t1 
             WHERE NOT EXISTS (
                 SELECT 1 FROM db2.{} t2 
                 WHERE t2.{} = t1.{}
             )",
            table_name, table_name, primary_key, primary_key
        ),
        [],
        |row| row.get(0)
    )?;
    
    // Find rows only in DB2 (inserted rows)
    let inserted_count: i64 = conn1.query_row(
        &format!(
            "SELECT COUNT(*) FROM db2.{} t2 
             WHERE NOT EXISTS (
                 SELECT 1 FROM {} t1 
                 WHERE t1.{} = t2.{}
             )",
            table_name, table_name, primary_key, primary_key
        ),
        [],
        |row| row.get(0)
    )?;
    
    // Find modified rows (exists in both but different)
    // This is an approximation - exact comparison would need column-by-column check
    let potentially_modified = (count1 - deleted_count).min(count2 - inserted_count);
    
    // Detach database
    conn1.execute("DETACH DATABASE db2", [])?;
    
    Ok(DataComparisonResult {
        table_name: table_name.to_string(),
        total_rows_db1: count1,
        total_rows_db2: count2,
        rows_inserted: inserted_count,
        rows_deleted: deleted_count,
        rows_potentially_modified: potentially_modified,
        identical: count1 == count2 && inserted_count == 0 && deleted_count == 0,
    })
}


    fn tables_are_identical(&self, table1: &TableInfo, table2: &TableInfo) -> bool {
        if table1.columns.len() != table2.columns.len() {
            return false;
        }

        for (col1, col2) in table1.columns.iter().zip(table2.columns.iter()) {
            if col1.name != col2.name || 
               col1.data_type != col2.data_type || 
               col1.is_nullable != col2.is_nullable ||
               col1.is_primary_key != col2.is_primary_key {
                return false;
            }
        }

        true
    }

    fn compare_tables(&self, table1: &TableInfo, table2: &TableInfo) -> TableDiff 
    {
        let col1_names: std::collections::HashSet<String> = table1.columns.iter().map(|c| c.name.clone()).collect();
        let col2_names: std::collections::HashSet<String> = table2.columns.iter().map(|c| c.name.clone()).collect();

        let added_columns: Vec<ColumnInfo> = table2.columns.iter()
            .filter(|c| !col1_names.contains(&c.name))
            .cloned()
            .collect();

        let removed_columns: Vec<String> = col1_names.difference(&col2_names)
            .cloned()
            .collect();

        // Build name->ColumnInfo maps
        let map1: std::collections::HashMap<_, _> = table1.columns.iter().map(|c| (c.name.clone(), c)).collect();
        let map2: std::collections::HashMap<_, _> = table2.columns.iter().map(|c| (c.name.clone(), c)).collect();

        // Columns that exist in both
        let common_cols: Vec<String> = col1_names.intersection(&col2_names).cloned().collect();

        let mut modified_columns = Vec::new();
        for name in common_cols {
            let c1 = map1.get(&name).unwrap();
            let c2 = map2.get(&name).unwrap();

            let mut changes = Vec::new();

            if c1.data_type.to_lowercase() != c2.data_type.to_lowercase() {
                changes.push(format!("type: {} -> {}", c1.data_type, c2.data_type));
            }
            if c1.is_nullable != c2.is_nullable {
                changes.push(format!("nullability: {} -> {}", c1.is_nullable, c2.is_nullable));
            }
            if c1.is_primary_key != c2.is_primary_key {
                changes.push(format!("primary key: {} -> {}", c1.is_primary_key, c2.is_primary_key));
            }
            if c1.default_value != c2.default_value {
                changes.push("default changed".to_string());
            }

            if !changes.is_empty() {
                modified_columns.push(ColumnDiff{
                    column_name: name,
                    old_type: c1.data_type.clone(),
                    new_type: c2.data_type.clone(),
                    changes,
                });
            }
        }


        TableDiff {
            table_name: table1.name.clone(),
            added_columns,
            removed_columns,
            modified_columns,
        }
    }
    
}