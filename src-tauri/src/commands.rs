use crate::database::DatabaseManager;
use crate::models::*;
use chrono::Local;
use rusqlite::Connection;
use std::io::Read;
use std::path::Path;
use std::sync::Mutex;
use std::time::Duration;
use tauri::State;
use tokio::time::timeout;

use crate::license::{LicenseManager, LicenseStatus};
use chrono::{DateTime, Utc};
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

// ADD THIS TYPE ALIAS AFTER YOUR IMPORTS
type LicenseManagerState = Mutex<LicenseManager>;
type DbManager = Mutex<DatabaseManager>;

#[derive(serde::Serialize)]
pub struct TrialInfo {
    is_expired: bool,
    remaining_days: i64,
    version: String,
}

#[tauri::command]
pub async fn connect_database(
    db_manager: State<'_, Mutex<DatabaseManager>>,
    path: String,
    password: String,
    settings: Option<serde_json::Value>, // Add this parameter
) -> Result<DatabaseInfo, String> {
    let mut manager = db_manager.lock().unwrap();

    // Parse settings if provided
    let sqlcipher_settings = if let Some(s) = settings {
        Some(s)
    } else {
        None
    };

    match manager.connect_database(&path, &password, sqlcipher_settings) {
        Ok(db_info) => {
            println!("✅ Connected to database: {}", path);
            Ok(db_info)
        }
        Err(e) => {
            println!("❌ Failed to connect: {}", e);
            Err(format!("Connection failed: {}", e))
        }
    }
}

#[tauri::command]
pub async fn get_table_info(db_path: String, table_name: String) -> Result<TableInfo, String> {
    use rusqlite::Connection;

    let conn = Connection::open(&db_path).map_err(|e| format!("Failed to open database: {}", e))?;

    let mut stmt = conn
        .prepare(&format!("PRAGMA table_info({})", table_name))
        .map_err(|e| format!("Failed to get table info: {}", e))?;

    let mut columns = Vec::new();
    let rows = stmt
        .query_map([], |row| {
            Ok(ColumnInfo {
                name: row.get(1)?,
                data_type: row.get(2)?,
                is_nullable: row.get::<_, i32>(3)? == 0,
                default_value: row.get(4).ok(),
                is_primary_key: row.get::<_, i32>(5)? == 1,
            })
        })
        .map_err(|e| format!("Query failed: {}", e))?;

    for row in rows {
        columns.push(row.map_err(|e| format!("Row error: {}", e))?);
    }

    let row_count: i64 = conn
        .query_row(&format!("SELECT COUNT(*) FROM {}", table_name), [], |row| {
            row.get(0)
        })
        .unwrap_or(0);

    Ok(TableInfo {
        name: table_name,
        row_count,
        columns,
    })
}

#[tauri::command]
pub async fn get_database_tables(
    db_path: String,
    manager: State<'_, DbManager>,
) -> Result<Vec<TableInfo>, String> {
    let db_manager = manager.lock().unwrap();

    match db_manager.get_tables(&db_path) {
        Ok(tables) => {
            println!("Retrieved {} tables from {}", tables.len(), db_path);
            Ok(tables)
        }
        Err(e) => {
            println!("Failed to get tables from {}: {}", db_path, e);
            Err(format!("Failed to get tables: {}", e))
        }
    }
}

#[tauri::command]
pub async fn get_table_data(
    db_path: String,
    table_name: String,
    limit: Option<i64>,
    offset: Option<i64>, // ← NEW PARAMETER
    manager: State<'_, DbManager>,
) -> Result<TableData, String> {
    // Add timeout for large queries (60 seconds)
    let timeout_duration = Duration::from_secs(60);

    let result = timeout(timeout_duration, async {
        let db_manager = manager.lock().unwrap();

        // Log the request with more details
        println!(
            "Fetching table '{}' from '{}' with limit: {:?}, offset: {:?}",
            table_name, db_path, limit, offset
        );

        // Check installation status for trial expiration
        let is_expired = check_installation_status().unwrap_or(false);

        // Enforce 2-row limit if expired
        let (effective_limit, effective_offset) = if is_expired {
            println!("Trial expired - enforcing 2 row limit for table data");
            (Some(2), Some(0)) // Force limit 2, offset 0
        } else {
            (limit, offset)
        };

        db_manager.get_table_data(&db_path, &table_name, effective_limit, effective_offset)
    })
    .await;

    match result {
        Ok(data_result) => match data_result {
            Ok(data) => {
                println!(
                    "Successfully retrieved {} rows from table '{}' in '{}'",
                    data.rows.len(),
                    table_name,
                    db_path
                );
                Ok(data)
            }
            Err(e) => {
                println!("Database error for table '{}': {}", table_name, e);
                Err(format!("Database error: {}", e))
            }
        },
        Err(_) => {
            println!(
                "Timeout occurred while fetching table '{}' (limit: {:?}, offset: {:?})",
                table_name, limit, offset
            );
            Err(format!(
                "Query timeout - table '{}' took too long to fetch. Try using a smaller row limit.",
                table_name
            ))
        }
    }
}
#[tauri::command]
pub async fn compare_database_schemas(
    db1_path: String,
    db2_path: String,
    manager: State<'_, DbManager>,
) -> Result<SchemaComparison, String> {
    let db_manager = manager.lock().unwrap();

    match db_manager.compare_schemas(&db1_path, &db2_path) {
        Ok(comparison) => {
            println!(
                "Schema comparison completed between {} and {}",
                db1_path, db2_path
            );
            Ok(comparison)
        }
        Err(e) => {
            println!("Failed to compare schemas: {}", e);
            Err(format!("Schema comparison failed: {}", e))
        }
    }
}

#[tauri::command]
pub async fn compare_table_data_fast(
    db1_path: String,
    db2_path: String,
    table_name: String,
    primary_key: String,
    manager: State<'_, DbManager>,
) -> Result<DataComparisonResult, String> {
    let db_manager = manager.lock().unwrap();

    println!("Fast comparing table '{}' between databases", table_name);

    match db_manager.compare_table_data_fast(&db1_path, &db2_path, &table_name, &primary_key) {
        Ok(result) => {
            println!(
                "Data comparison completed: {} total rows",
                result.total_rows_db1
            );
            Ok(result)
        }
        Err(e) => {
            println!("Failed to compare data: {}", e);
            Err(format!("Data comparison failed: {}", e))
        }
    }
}

#[tauri::command]
pub async fn test_connection() -> Result<String, String> {
    Ok("Tauri backend is working!".to_string())
}

#[tauri::command]
pub async fn generate_schema_patch(
    db1_path: String,
    db2_path: String,
    direction: Option<String>, // "source_to_target" (default) or "target_to_source"
    db_manager: State<'_, Mutex<DatabaseManager>>,
) -> Result<String, String> {
    let manager = db_manager.lock().unwrap();

    let comparison = manager
        .compare_schemas(&db1_path, &db2_path)
        .map_err(|e| e.to_string())?;

    // Determine direction
    let is_reverse = direction.as_deref() == Some("target_to_source");

    // Open BOTH databases to get schemas
    let source_conn = Connection::open(&db1_path)
        .map_err(|e| format!("Failed to open source database: {}", e))?;

    let target_conn = Connection::open(&db2_path)
        .map_err(|e| format!("Failed to open target database: {}", e))?;

    // Get passwords and unlock if needed
    let db1_password = manager.get_password(&db1_path).unwrap_or_default();
    let db2_password = manager.get_password(&db2_path).unwrap_or_default();

    unlock_database(&source_conn, &db1_password)?;
    unlock_database(&target_conn, &db2_password)?;

    // Generate the SQL patch
    let mut sql = String::new();
    sql.push_str("-- Schema Migration Patch\n");
    if is_reverse {
        sql.push_str("-- Direction: Target → Source (Reverse)\n");
        sql.push_str(&format!("-- Template: {} (Target)\n", comparison.database2));
        sql.push_str(&format!("-- Apply to: {} (Source)\n", comparison.database1));
    } else {
        sql.push_str("-- Direction: Source → Target (Forward)\n");
        sql.push_str(&format!("-- Template: {} (Source)\n", comparison.database1));
        sql.push_str(&format!("-- Apply to: {} (Target)\n", comparison.database2));
    }
    sql.push_str(&format!("-- Generated: {}\n\n", chrono::Utc::now()));
    sql.push_str("-- NOTE: PRAGMA statements require the transaction to be committed first.\n");
    sql.push_str("-- If running in DB Browser, close any open transactions before executing.\n\n");

    // Track if we need PRAGMA statements (table recreation)
    let has_table_recreation = comparison.modified_tables.iter().any(|m| {
        !m.removed_columns.is_empty()
            || !m.added_columns.is_empty()
            || !m.modified_columns.is_empty()
    });

    // Start transaction only for simple operations (no PRAGMA needed)
    if !has_table_recreation {
        sql.push_str("BEGIN TRANSACTION;\n\n");
    }

    if is_reverse {
        // REVERSE: Make SOURCE match TARGET
        // Create tables that exist in TARGET but not in SOURCE
        for table in &comparison.added_tables {
            match get_create_table_sql(&target_conn, table) {
                Ok(create_sql) => {
                    sql.push_str("-- Create table from target\n");
                    sql.push_str(&format!("{};\n\n", create_sql));
                }
                Err(e) => {
                    sql.push_str(&format!(
                        "-- ERROR: Could not get schema for {}: {}\n\n",
                        table, e
                    ));
                }
            }
        }

        // Drop tables that exist in SOURCE but not in TARGET
        for table in &comparison.removed_tables {
            sql.push_str("-- Drop table not in target\n");
            sql.push_str(&format!("DROP TABLE IF EXISTS `{}`;\n\n", table));
        }
    } else {
        // FORWARD: Make TARGET match SOURCE (original behavior)
        // Create tables that exist in SOURCE but not in TARGET
        for table in &comparison.removed_tables {
            match get_create_table_sql(&source_conn, table) {
                Ok(create_sql) => {
                    sql.push_str("-- Create table from source\n");
                    sql.push_str(&format!("{};\n\n", create_sql));
                }
                Err(e) => {
                    sql.push_str(&format!(
                        "-- ERROR: Could not get schema for {}: {}\n\n",
                        table, e
                    ));
                }
            }
        }

        // Drop tables that exist in TARGET but not in SOURCE
        for table in &comparison.added_tables {
            sql.push_str("-- Drop table not in source\n");
            sql.push_str(&format!("DROP TABLE IF EXISTS `{}`;\n\n", table));
        }
    }

    // Modify existing tables
    for modified in &comparison.modified_tables {
        // added_columns = columns in TARGET but not in SOURCE
        // removed_columns = columns in SOURCE but not in TARGET
        // modified_columns = columns with different types/constraints

        if is_reverse {
            // REVERSE: Make SOURCE match TARGET
            let needs_recreation =
                !modified.removed_columns.is_empty() || !modified.modified_columns.is_empty();

            if needs_recreation {
                sql.push_str(&format!(
                    "-- Recreate table: {} (reverse direction)\n",
                    modified.table_name
                ));
                match generate_table_recreation_sql_reverse(
                    &target_conn,
                    &source_conn,
                    &modified.table_name,
                    &modified,
                ) {
                    Ok(recreation_sql) => {
                        sql.push_str(&recreation_sql);
                    }
                    Err(e) => {
                        sql.push_str(&format!(
                            "-- ERROR: Could not generate recreation SQL for {}: {}\n",
                            modified.table_name, e
                        ));
                    }
                }
                sql.push_str("\n");
            } else if !modified.added_columns.is_empty() {
                sql.push_str(&format!(
                    "-- Modify table: {} (add columns from target)\n",
                    modified.table_name
                ));
                for col_name in &modified.added_columns {
                    match get_column_info(&target_conn, &modified.table_name, &col_name.name) {
                        Ok(col) => {
                            let nullable = if col.is_nullable { "" } else { " NOT NULL" };
                            let default = match &col.default_value {
                                Some(def) => format!(" DEFAULT {}", def),
                                None => String::new(),
                            };
                            sql.push_str(&format!(
                                "ALTER TABLE `{}` ADD COLUMN `{}` {}{}{};\n",
                                modified.table_name, col.name, col.data_type, nullable, default
                            ));
                        }
                        Err(e) => {
                            sql.push_str(&format!(
                                "-- ERROR: Could not get column info for {}: {}\n",
                                col_name.name, e
                            ));
                        }
                    }
                }
                sql.push_str("\n");
            }
        } else {
            // FORWARD: Make TARGET match SOURCE (original behavior)
            let needs_recreation =
                !modified.added_columns.is_empty() || !modified.modified_columns.is_empty();

            if needs_recreation {
                sql.push_str(&format!(
                    "-- Recreate table: {} (columns removed/modified)\n",
                    modified.table_name
                ));
                match generate_table_recreation_sql(
                    &source_conn,
                    &target_conn,
                    &modified.table_name,
                    &modified,
                ) {
                    Ok(recreation_sql) => {
                        sql.push_str(&recreation_sql);
                    }
                    Err(e) => {
                        sql.push_str(&format!(
                            "-- ERROR: Could not generate recreation SQL for {}: {}\n",
                            modified.table_name, e
                        ));
                        sql.push_str(&format!("-- Manual recreation required for:\n"));
                        if !modified.added_columns.is_empty() {
                            sql.push_str(&format!(
                                "--   Columns to drop: {}\n",
                                modified
                                    .added_columns
                                    .iter()
                                    .map(|c| c.name.as_str())
                                    .collect::<Vec<_>>()
                                    .join(", ")
                            ));
                        }
                        for mod_col in &modified.modified_columns {
                            sql.push_str(&format!(
                                "--   Modified column: {} ({} -> {})\n",
                                mod_col.column_name, mod_col.old_type, mod_col.new_type
                            ));
                        }
                    }
                }
                sql.push_str("\n");
            } else if !modified.removed_columns.is_empty() {
                sql.push_str(&format!(
                    "-- Modify table: {} (add columns from source)\n",
                    modified.table_name
                ));
                for col_name in &modified.removed_columns {
                    match get_column_info(&source_conn, &modified.table_name, col_name) {
                        Ok(col) => {
                            let nullable = if col.is_nullable { "" } else { " NOT NULL" };
                            let default = match &col.default_value {
                                Some(def) => format!(" DEFAULT {}", def),
                                None => String::new(),
                            };
                            sql.push_str(&format!(
                                "ALTER TABLE `{}` ADD COLUMN `{}` {}{}{};\n",
                                modified.table_name, col.name, col.data_type, nullable, default
                            ));
                        }
                        Err(e) => {
                            sql.push_str(&format!(
                                "-- ERROR: Could not get column info for {}: {}\n",
                                col_name, e
                            ));
                        }
                    }
                }
                sql.push_str("\n");
            }
        }
    }

    // Close transaction only if we started one
    if !has_table_recreation {
        sql.push_str("COMMIT;\n");
    }

    sql.push_str("\n-- Migration complete\n");

    Ok(sql)
}

#[tauri::command]
pub async fn apply_patch_file(
    target_db_path: String,
    patch_file_path: String,
    db_manager: State<'_, Mutex<DatabaseManager>>,
) -> Result<String, String> {
    // Read the file content
    let patch_sql = std::fs::read_to_string(&patch_file_path)
        .map_err(|e| format!("Failed to read patch file: {}", e))?;

    // Reuse the existing apply logic
    // We can just call apply_schema_patch since we have the content now
    // and loading 100-200MB into RAM on backend is fine (unlike frontend)
    apply_schema_patch(target_db_path, patch_sql, db_manager).await
}

#[tauri::command]
pub async fn save_temp_file(temp_path: String, dest_path: String) -> Result<(), String> {
    std::fs::copy(&temp_path, &dest_path)
        .map(|_| ())
        .map_err(|e| format!("Failed to save file: {}", e))
}

#[tauri::command]
pub fn check_installation_status() -> Result<bool, String> {
    // Hidden obfuscated file path: ~/.local/share/.plandb-data/sys_core.bin
    let home_dir =
        std::env::var("HOME").map_err(|_| "Could not find HOME directory".to_string())?;
    let data_dir = Path::new(&home_dir)
        .join(".local")
        .join("share")
        .join(".plandb-data");
    let sys_file = data_dir.join("sys_core.bin");

    // Ensure directory exists
    if !data_dir.exists() {
        fs::create_dir_all(&data_dir)
            .map_err(|e| format!("Failed to create data directory: {}", e))?;
    }

    if !sys_file.exists() {
        // First run: Create file with current timestamp
        let start = SystemTime::now();
        let since_the_epoch = start
            .duration_since(UNIX_EPOCH)
            .map_err(|e| format!("Time error: {}", e))?;
        let timestamp = since_the_epoch.as_secs();

        // Obfuscate: XOR with a key (simple but effective against casual editing)
        let key: u64 = 0xDEADBEEFCAFEBABE;
        let obfuscated = timestamp ^ key;

        let bytes = obfuscated.to_le_bytes();
        let mut file = fs::File::create(&sys_file)
            .map_err(|e| format!("Failed to create system file: {}", e))?;
        file.write_all(&bytes)
            .map_err(|e| format!("Failed to write system file: {}", e))?;

        println!("System initialized. Trial started.");
        return Ok(false); // Not expired
    }

    // Read and verify
    let bytes = fs::read(&sys_file).map_err(|e| format!("Failed to read system file: {}", e))?;
    if bytes.len() != 8 {
        // Corrupted file? Reset it (or could treat as expired/tampered)
        println!("System file corrupted. Resetting.");
        let _ = fs::remove_file(&sys_file);
        return check_installation_status();
    }

    let mut arr = [0u8; 8];
    arr.copy_from_slice(&bytes);
    let obfuscated = u64::from_le_bytes(arr);

    let key: u64 = 0xDEADBEEFCAFEBABE;
    let timestamp = obfuscated ^ key;

    let install_time = UNIX_EPOCH + Duration::from_secs(timestamp);
    let now = SystemTime::now();

    let duration = now
        .duration_since(install_time)
        .unwrap_or(Duration::from_secs(0));
    let days = duration.as_secs() / 86400;

    println!("System active for {} days", days);

    if days > 90 {
        Ok(true) // Expired
    } else {
        Ok(false) // Active
    }
}

#[tauri::command]
pub fn get_trial_info() -> Result<TrialInfo, String> {
    // Hidden obfuscated file path: ~/.local/share/.plandb-data/sys_core.bin
    let home_dir =
        std::env::var("HOME").map_err(|_| "Could not find HOME directory".to_string())?;
    let data_dir = Path::new(&home_dir)
        .join(".local")
        .join("share")
        .join(".plandb-data");
    let sys_file = data_dir.join("sys_core.bin");

    // Ensure directory exists
    if !data_dir.exists() {
        fs::create_dir_all(&data_dir)
            .map_err(|e| format!("Failed to create data directory: {}", e))?;
    }

    let timestamp = if !sys_file.exists() {
        // First run: Create file with current timestamp
        let start = SystemTime::now();
        let since_the_epoch = start
            .duration_since(UNIX_EPOCH)
            .map_err(|e| format!("Time error: {}", e))?;
        let ts = since_the_epoch.as_secs();

        // Obfuscate: XOR with a key
        let key: u64 = 0xDEADBEEFCAFEBABE;
        let obfuscated = ts ^ key;

        let bytes = obfuscated.to_le_bytes();
        let mut file = fs::File::create(&sys_file)
            .map_err(|e| format!("Failed to create system file: {}", e))?;
        file.write_all(&bytes)
            .map_err(|e| format!("Failed to write system file: {}", e))?;

        ts
    } else {
        // Read and verify
        let bytes =
            fs::read(&sys_file).map_err(|e| format!("Failed to read system file: {}", e))?;
        if bytes.len() != 8 {
            // Corrupted file? Reset it
            let _ = fs::remove_file(&sys_file);
            // Recursively call to recreate
            return get_trial_info();
        }

        let mut arr = [0u8; 8];
        arr.copy_from_slice(&bytes);
        let obfuscated = u64::from_le_bytes(arr);

        let key: u64 = 0xDEADBEEFCAFEBABE;
        obfuscated ^ key
    };

    let install_time = UNIX_EPOCH + Duration::from_secs(timestamp);
    let now = SystemTime::now();

    let duration = now
        .duration_since(install_time)
        .unwrap_or(Duration::from_secs(0));
    let active_days = duration.as_secs() / 86400;

    let remaining_days = if active_days >= 90 {
        0
    } else {
        90 - active_days
    };

    Ok(TrialInfo {
        is_expired: active_days > 90,
        remaining_days: remaining_days as i64,
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}

#[tauri::command]
pub async fn apply_schema_patch(
    target_db_path: String,
    patch_sql: String,
    db_manager: State<'_, Mutex<DatabaseManager>>,
) -> Result<String, String> {
    const BATCH_SIZE: usize = 500; // Commit every 500 statements to prevent long locks

    let db_password = {
        let manager = db_manager.lock().unwrap();
        manager.get_password(&target_db_path).unwrap_or_default()
    };

    // Open target database with a new connection
    let conn = Connection::open(&target_db_path)
        .map_err(|e| format!("Failed to open target database: {}", e))?;

    // Unlock if needed
    unlock_database(&conn, &db_password)?;

    // Prepare statements - filter and clean first
    let all_statements: Vec<&str> = patch_sql.split(';').collect();
    let mut cleaned_statements: Vec<String> = Vec::new();

    for statement in all_statements.iter() {
        let trimmed = statement.trim();
        if trimmed.is_empty() {
            continue;
        }

        // Do NOT remove comment lines blindly!
        // This corrupts data that contains lines starting with '--' (e.g. separator lines in text)
        // SQLite handles comments natively, so we can pass them through.

        if !trimmed.is_empty() {
            cleaned_statements.push(trimmed.to_string());
        }
    }

    let total_statements = cleaned_statements.len();
    let mut executed = 0;

    // Start initial transaction
    conn.execute("BEGIN IMMEDIATE", [])
        .map_err(|e| format!("Failed to begin transaction: {}", e))?;

    // Execute statements in batches
    for (idx, statement) in cleaned_statements.iter().enumerate() {
        // Check if it's a transaction command (ignoring comments)
        // We must skip BEGIN/COMMIT from the patch because we manage transactions manually
        let uncommented = statement
            .lines()
            .filter(|line| !line.trim().starts_with("--"))
            .collect::<Vec<_>>()
            .join("\n");

        let stmt_upper = uncommented.trim().to_uppercase();

        // Skip transaction commands
        if stmt_upper.starts_with("BEGIN") || stmt_upper.starts_with("COMMIT") {
            continue;
        }

        // Skip comment-only statements (after removing comment lines, nothing is left)
        if stmt_upper.is_empty() {
            continue;
        }

        // Execute the statement (original statement with comments preserved)
        conn.execute(statement, []).map_err(|e| {
            // Try to rollback on error
            let _ = conn.execute("ROLLBACK", []);
            format!(
                "Error at statement {}/{}: {}\nStatement: {}",
                idx + 1,
                total_statements,
                e,
                statement
            )
        })?;

        executed += 1;

        // Commit and start new transaction every BATCH_SIZE statements
        // This prevents long-running transactions that lock the database
        if executed % BATCH_SIZE == 0 && idx < cleaned_statements.len() - 1 {
            conn.execute("COMMIT", [])
                .map_err(|e| format!("Failed to commit batch: {}", e))?;

            // Yield to system to prevent UI freeze
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

            conn.execute("BEGIN IMMEDIATE", [])
                .map_err(|e| format!("Failed to begin new batch: {}", e))?;
        }
    }

    // Final commit
    conn.execute("COMMIT", [])
        .map_err(|e| format!("Failed to final commit: {}", e))?;

    // Close the connection to flush changes
    drop(conn);

    // Force DatabaseManager to reconnect to see the updated schema
    {
        let mut manager = db_manager.lock().unwrap();
        // Disconnect and reconnect to refresh the schema cache
        manager.disconnect(&target_db_path);
        let _ = manager.connect_database(&target_db_path, &db_password, None);
    }

    Ok(format!(
        "Schema patch applied successfully. Executed {} statements.",
        executed
    ))
}

// Helper function to unlock database
fn unlock_database(conn: &Connection, password: &str) -> Result<(), String> {
    if !password.is_empty() {
        let key_formats = vec![
            format!("PRAGMA key = '{}';", password),
            format!("PRAGMA key = \"{}\";", password),
            format!("PRAGMA key = {};", password),
            format!("PRAGMA key='{}';", password),
        ];

        for key_format in key_formats {
            if conn.execute_batch(&key_format).is_ok() {
                if conn
                    .query_row(
                        "SELECT COUNT(*) FROM sqlite_master WHERE type='table';",
                        [],
                        |_| Ok(()),
                    )
                    .is_ok()
                {
                    return Ok(());
                }
            }
        }
        return Err("Failed to unlock database with provided password".to_string());
    }

    // Verify unencrypted database is readable
    conn.query_row(
        "SELECT COUNT(*) FROM sqlite_master WHERE type='table';",
        [],
        |_| Ok(()),
    )
    .map_err(|e| format!("Failed to read database: {}", e))?;

    Ok(())
}

// Generate table recreation SQL (12-step process)
fn generate_table_recreation_sql(
    source_conn: &Connection,
    target_conn: &Connection,
    table_name: &str,
    diff: &crate::models::TableDiff,
) -> Result<String, String> {
    // Get the SOURCE schema (what we want - make target match source)
    let mut source_schema = get_create_table_sql(source_conn, table_name)
        .map_err(|e| format!("Could not get source schema: {}", e))?;

    // Fix unsupported collations
    source_schema = source_schema
        .replace("COLLATE UTF16", "COLLATE BINARY")
        .replace("COLLATE utf16", "COLLATE BINARY")
        .replace("COLLATE UTF8", "COLLATE BINARY")
        .replace("COLLATE utf8", "COLLATE BINARY");

    // Get columns from source (final state - what we want)
    let source_columns = get_table_column_names(source_conn, table_name)?;

    // Get columns from target (current state)
    let target_columns = get_table_column_names(target_conn, table_name)?;

    // Find common columns for data migration
    let common_columns: Vec<String> = source_columns
        .iter()
        .filter(|col| target_columns.contains(col))
        .cloned()
        .collect();

    if common_columns.is_empty() {
        return Err("No common columns found - table would lose all data".to_string());
    }

    let common_cols_str = common_columns
        .iter()
        .map(|c| format!("`{}`", c))
        .collect::<Vec<_>>()
        .join(", ");

    let mut sql = String::new();

    // Step 1: Disable foreign keys (must be outside transaction)
    sql.push_str("PRAGMA foreign_keys=off;\n\n");

    // Step 2: Start a transaction for this table recreation
    sql.push_str("BEGIN IMMEDIATE;\n\n");

    // Step 3: Rename old table (current target)
    sql.push_str(&format!(
        "ALTER TABLE `{}` RENAME TO `{}_old`;\n\n",
        table_name, table_name
    ));

    // Step 4: Create new table with source schema (make target match source)
    sql.push_str(&format!("{};\n\n", source_schema));

    // Step 5: Copy data from old to new table
    sql.push_str(&format!(
        "INSERT INTO `{}` ({})\nSELECT {}\nFROM `{}_old`;\n\n",
        table_name, common_cols_str, common_cols_str, table_name
    ));

    // Step 6: Drop old table
    sql.push_str(&format!("DROP TABLE `{}_old`;\n\n", table_name));

    // Step 7: Commit this table's transaction
    sql.push_str("COMMIT;\n\n");

    // Step 8: Re-enable foreign keys (must be outside transaction)
    sql.push_str("PRAGMA foreign_keys=on;\n\n");

    // Add comment about what changed
    sql.push_str("-- Changes applied:\n");
    if !diff.removed_columns.is_empty() {
        sql.push_str(&format!(
            "--   Added (from source): {}\n",
            diff.removed_columns.join(", ")
        ));
    }
    if !diff.added_columns.is_empty() {
        let dropped: Vec<String> = diff.added_columns.iter().map(|c| c.name.clone()).collect();
        sql.push_str(&format!(
            "--   Dropped (not in source): {}\n",
            dropped.join(", ")
        ));
    }
    if !diff.modified_columns.is_empty() {
        for mod_col in &diff.modified_columns {
            sql.push_str(&format!(
                "--   Modified: {} ({} -> {})\n",
                mod_col.column_name, mod_col.old_type, mod_col.new_type
            ));
        }
    }

    Ok(sql)
}

// Generate table recreation SQL for REVERSE direction (make source match target)
fn generate_table_recreation_sql_reverse(
    target_conn: &Connection,
    source_conn: &Connection,
    table_name: &str,
    diff: &crate::models::TableDiff,
) -> Result<String, String> {
    // Get the TARGET schema (what we want - make source match target)
    let mut target_schema = get_create_table_sql(target_conn, table_name)
        .map_err(|e| format!("Could not get target schema: {}", e))?;

    // Fix unsupported collations
    target_schema = target_schema
        .replace("COLLATE UTF16", "COLLATE BINARY")
        .replace("COLLATE utf16", "COLLATE BINARY")
        .replace("COLLATE UTF8", "COLLATE BINARY")
        .replace("COLLATE utf8", "COLLATE BINARY");

    // Get columns from target (final state - what we want)
    let target_columns = get_table_column_names(target_conn, table_name)?;

    // Get columns from source (current state)
    let source_columns = get_table_column_names(source_conn, table_name)?;

    // Find common columns for data migration
    let common_columns: Vec<String> = target_columns
        .iter()
        .filter(|col| source_columns.contains(col))
        .cloned()
        .collect();

    if common_columns.is_empty() {
        return Err("No common columns found - table would lose all data".to_string());
    }

    let common_cols_str = common_columns
        .iter()
        .map(|c| format!("`{}`", c))
        .collect::<Vec<_>>()
        .join(", ");

    let mut sql = String::new();

    // Step 1: Disable foreign keys
    sql.push_str("PRAGMA foreign_keys=off;\n\n");

    // Step 2: Start transaction
    sql.push_str("BEGIN IMMEDIATE;\n\n");

    // Step 3: Rename old table (current source)
    sql.push_str(&format!(
        "ALTER TABLE `{}` RENAME TO `{}_old`;\n\n",
        table_name, table_name
    ));

    // Step 4: Create new table with target schema (make source match target)
    sql.push_str(&format!("{};\n\n", target_schema));

    // Step 5: Copy data from old to new table
    sql.push_str(&format!(
        "INSERT INTO `{}` ({})\nSELECT {}\nFROM `{}_old`;\n\n",
        table_name, common_cols_str, common_cols_str, table_name
    ));

    // Step 6: Drop old table
    sql.push_str(&format!("DROP TABLE `{}_old`;\n\n", table_name));

    // Step 7: Commit transaction
    sql.push_str("COMMIT;\n\n");

    // Step 8: Re-enable foreign keys
    sql.push_str("PRAGMA foreign_keys=on;\n\n");

    // Add comment about what changed
    sql.push_str("-- Changes applied (reverse direction):\n");
    if !diff.added_columns.is_empty() {
        let added: Vec<String> = diff.added_columns.iter().map(|c| c.name.clone()).collect();
        sql.push_str(&format!("--   Added (from target): {}\n", added.join(", ")));
    }
    if !diff.removed_columns.is_empty() {
        sql.push_str(&format!(
            "--   Dropped (not in target): {}\n",
            diff.removed_columns.join(", ")
        ));
    }
    if !diff.modified_columns.is_empty() {
        for mod_col in &diff.modified_columns {
            sql.push_str(&format!(
                "--   Modified: {} ({} -> {})\n",
                mod_col.column_name, mod_col.new_type, mod_col.old_type
            ));
        }
    }

    Ok(sql)
}

// Get column names from a table
fn get_table_column_names(conn: &Connection, table_name: &str) -> Result<Vec<String>, String> {
    let mut stmt = conn
        .prepare(&format!("PRAGMA table_info(`{}`)", table_name))
        .map_err(|e| format!("Failed to get table info: {}", e))?;

    let columns: Vec<String> = stmt
        .query_map([], |row| {
            row.get::<_, String>(1) // Column name is at index 1
        })
        .map_err(|e| format!("Failed to query columns: {}", e))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("Failed to collect columns: {}", e))?;

    Ok(columns)
}

fn get_create_table_sql(conn: &Connection, table_name: &str) -> Result<String, rusqlite::Error> {
    conn.query_row(
        "SELECT sql FROM sqlite_master WHERE type='table' AND name=?1",
        [table_name],
        |row| row.get(0),
    )
}

// Helper function to get column info for a specific column
fn get_column_info(
    conn: &Connection,
    table_name: &str,
    column_name: &str,
) -> Result<ColumnInfo, String> {
    let mut stmt = conn
        .prepare(&format!("PRAGMA table_info(`{}`)", table_name))
        .map_err(|e| format!("Failed to get table info: {}", e))?;

    let column = stmt
        .query_map([], |row| {
            Ok(ColumnInfo {
                name: row.get(1)?,
                data_type: row.get(2)?,
                is_nullable: row.get::<_, i32>(3)? == 0,
                default_value: row.get(4).ok(),
                is_primary_key: row.get::<_, i32>(5)? == 1,
            })
        })
        .map_err(|e| format!("Failed to query columns: {}", e))?
        .find_map(|col_result| {
            if let Ok(col) = col_result {
                if col.name == column_name {
                    return Some(col);
                }
            }
            None
        })
        .ok_or_else(|| {
            format!(
                "Column '{}' not found in table '{}'",
                column_name, table_name
            )
        })?;

    Ok(column)
}

/// Detect if added columns require table recreation (middle insertion)
/// Returns true if any added column needs to be inserted before the last column
fn needs_column_order_recreation(
    source_conn: &Connection,
    target_conn: &Connection,
    table_name: &str,
    added_columns: &[ColumnInfo],
) -> Result<bool, String> {
    if added_columns.is_empty() {
        return Ok(false);
    }

    // Get source column count
    let source_col_count: i32 = source_conn
        .prepare(&format!("PRAGMA table_info(`{}`)", table_name))
        .map_err(|e| format!("Failed to get source table info: {}", e))?
        .query_map([], |_| Ok(()))
        .map_err(|e| format!("Failed to count source columns: {}", e))?
        .count() as i32;

    // Get all target columns with positions
    let mut target_columns: Vec<(String, i32)> = target_conn
        .prepare(&format!("PRAGMA table_info(`{}`)", table_name))
        .map_err(|e| format!("Failed to get target table info: {}", e))?
        .query_map([], |row| {
            Ok((
                row.get::<_, String>(1)?, // column name
                row.get::<_, i32>(0)?,    // cid (position)
            ))
        })
        .map_err(|e| format!("Failed to query target columns: {}", e))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("Failed to collect target columns: {}", e))?;

    // Check if any added column has position < source_col_count
    // If so, it needs middle insertion
    for added_col in added_columns {
        if let Some((_, position)) = target_columns
            .iter()
            .find(|(name, _)| name == &added_col.name)
        {
            if *position < source_col_count {
                // This column needs to be inserted in the middle
                return Ok(true);
            }
        }
    }

    // All added columns are at the end, can use ALTER TABLE
    Ok(false)
}

#[tauri::command]
pub async fn generate_table_schema_patch(
    db1_path: String,
    db2_path: String,
    table_name: String,
    table_status: String,      // "added", "removed", or "modified"
    direction: Option<String>, // "source_to_target" (default) or "target_to_source"
    db_manager: State<'_, Mutex<DatabaseManager>>,
) -> Result<String, String> {
    let manager = db_manager.lock().unwrap();

    // Determine direction
    let is_reverse = direction.as_deref() == Some("target_to_source");

    // Open BOTH databases
    let source_conn = Connection::open(&db1_path)
        .map_err(|e| format!("Failed to open source database: {}", e))?;

    let target_conn = Connection::open(&db2_path)
        .map_err(|e| format!("Failed to open target database: {}", e))?;

    // Get passwords and unlock if needed
    let db1_password = manager.get_password(&db1_path).unwrap_or_default();
    let db2_password = manager.get_password(&db2_path).unwrap_or_default();

    unlock_database(&source_conn, &db1_password)?;
    unlock_database(&target_conn, &db2_password)?;

    // Generate patch header
    let mut sql = String::new();
    sql.push_str(&format!("-- Single Table Schema Patch: {}\n", table_name));
    if is_reverse {
        sql.push_str("-- Direction: Target → Source (Reverse)\n");
        sql.push_str(&format!("-- Apply to: {} (Source)\n", db1_path));
    } else {
        sql.push_str("-- Direction: Source → Target (Forward)\n");
        sql.push_str(&format!("-- Apply to: {} (Target)\n", db2_path));
    }
    sql.push_str(&format!("-- Generated: {}\n\n", chrono::Utc::now()));

    // Generate SQL based on table status and direction
    match table_status.as_str() {
        "added" => {
            // Added table: exists in TARGET but not in SOURCE
            if is_reverse {
                // Reverse: Create in SOURCE (from TARGET schema)
                sql.push_str("-- Create table from target database\n");
                match get_create_table_sql(&target_conn, &table_name) {
                    Ok(create_sql) => {
                        sql.push_str(&format!("{};\n", create_sql));
                    }
                    Err(e) => {
                        return Err(format!("Could not get table schema: {}", e));
                    }
                }
            } else {
                // Forward: Drop from TARGET
                sql.push_str("-- Drop table from target database\n");
                sql.push_str(&format!("DROP TABLE IF EXISTS `{}`;\n", table_name));
            }
        }
        "removed" => {
            // Removed table: exists in SOURCE but not in TARGET
            if is_reverse {
                // Reverse: Drop from SOURCE
                sql.push_str("-- Drop table from source database\n");
                sql.push_str(&format!("DROP TABLE IF EXISTS `{}`;\n", table_name));
            } else {
                // Forward: Create in TARGET (from SOURCE schema)
                sql.push_str("-- Create table from source database\n");
                match get_create_table_sql(&source_conn, &table_name) {
                    Ok(create_sql) => {
                        sql.push_str(&format!("{};\n", create_sql));
                    }
                    Err(e) => {
                        return Err(format!("Could not get table schema: {}", e));
                    }
                }
            }
        }
        "modified" => {
            // Modified table: needs comparison to determine changes
            let comparison = manager
                .compare_schemas(&db1_path, &db2_path)
                .map_err(|e| e.to_string())?;

            // Find the specific table diff
            let table_diff = comparison
                .modified_tables
                .iter()
                .find(|t| t.table_name == table_name)
                .ok_or_else(|| format!("Table '{}' not found in modified tables", table_name))?;

            if is_reverse {
                // Reverse: Make SOURCE match TARGET
                let needs_recreation = !table_diff.removed_columns.is_empty()
                    || !table_diff.modified_columns.is_empty();

                if needs_recreation {
                    sql.push_str("-- Recreate table to match target schema\n");
                    match generate_table_recreation_sql_reverse(
                        &target_conn,
                        &source_conn,
                        &table_name,
                        table_diff,
                    ) {
                        Ok(recreation_sql) => {
                            sql.push_str(&recreation_sql);
                        }
                        Err(e) => {
                            return Err(format!("Could not generate recreation SQL: {}", e));
                        }
                    }
                } else if !table_diff.added_columns.is_empty() {
                    // Check if columns need middle insertion (smart detection)
                    let needs_recreation = needs_column_order_recreation(
                        &source_conn,
                        &target_conn,
                        &table_name,
                        &table_diff.added_columns,
                    )?;

                    if needs_recreation {
                        // Use table recreation to preserve column order
                        sql.push_str("-- Table recreation required (column order preservation)\n");
                        match generate_table_recreation_sql_reverse(
                            &target_conn,
                            &source_conn,
                            &table_name,
                            table_diff,
                        ) {
                            Ok(recreation_sql) => {
                                sql.push_str(&recreation_sql);
                            }
                            Err(e) => {
                                return Err(format!("Could not generate recreation SQL: {}", e));
                            }
                        }
                    } else {
                        // All columns at end, use simple ALTER TABLE
                        sql.push_str(&format!("-- Add columns from target (at end)\n"));
                        for col_name in &table_diff.added_columns {
                            match get_column_info(&target_conn, &table_name, &col_name.name) {
                                Ok(col) => {
                                    let nullable = if col.is_nullable { "" } else { " NOT NULL" };
                                    let default = match &col.default_value {
                                        Some(def) => format!(" DEFAULT {}", def),
                                        None => String::new(),
                                    };
                                    sql.push_str(&format!(
                                        "ALTER TABLE `{}` ADD COLUMN `{}` {}{}{};\n",
                                        table_name, col.name, col.data_type, nullable, default
                                    ));
                                }
                                Err(e) => {
                                    return Err(format!("Could not get column info: {}", e));
                                }
                            }
                        }
                    }
                }
            } else {
                // Forward: Make TARGET match SOURCE
                // In forward direction:
                // - added_columns = columns in TARGET not in SOURCE (need to be DROPPED from target)
                // - removed_columns = columns in SOURCE not in TARGET (need to be ADDED to target)

                // Check if we need recreation for any reason:
                // 1. Modifications exist
                // 2. Need to drop columns (added_columns)
                // 3. Need to add columns in middle (removed_columns with position check)

                let has_modifications = !table_diff.modified_columns.is_empty();
                let needs_drop_columns = !table_diff.added_columns.is_empty();

                // Check if removed_columns (adding to target) need middle insertion
                let columns_to_add: Vec<ColumnInfo> = table_diff
                    .removed_columns
                    .iter()
                    .filter_map(|col_name| {
                        get_column_info(&source_conn, &table_name, col_name).ok()
                    })
                    .collect();

                let needs_order_preservation = if !columns_to_add.is_empty() {
                    needs_column_order_recreation(
                        &target_conn,
                        &source_conn,
                        &table_name,
                        &columns_to_add,
                    )?
                } else {
                    false
                };

                let needs_recreation =
                    has_modifications || needs_drop_columns || needs_order_preservation;

                if needs_recreation {
                    if needs_drop_columns && !needs_order_preservation && !has_modifications {
                        sql.push_str("-- Table recreation required (drop columns)\n");
                    } else if needs_order_preservation {
                        sql.push_str("-- Table recreation required (column order preservation)\n");
                    } else {
                        sql.push_str("-- Recreate table to match source schema\n");
                    }
                    match generate_table_recreation_sql(
                        &source_conn,
                        &target_conn,
                        &table_name,
                        table_diff,
                    ) {
                        Ok(recreation_sql) => {
                            sql.push_str(&recreation_sql);
                        }
                        Err(e) => {
                            return Err(format!("Could not generate recreation SQL: {}", e));
                        }
                    }
                } else if !table_diff.removed_columns.is_empty() {
                    // All columns at end, use simple ALTER TABLE
                    sql.push_str(&format!("-- Add columns from source (at end)\n"));
                    for col_name in &table_diff.removed_columns {
                        match get_column_info(&source_conn, &table_name, col_name) {
                            Ok(col) => {
                                let nullable = if col.is_nullable { "" } else { " NOT NULL" };
                                let default = match &col.default_value {
                                    Some(def) => format!(" DEFAULT {}", def),
                                    None => String::new(),
                                };
                                sql.push_str(&format!(
                                    "ALTER TABLE `{}` ADD COLUMN `{}` {}{}{};\n",
                                    table_name, col.name, col.data_type, nullable, default
                                ));
                            }
                            Err(e) => {
                                return Err(format!("Could not get column info: {}", e));
                            }
                        }
                    }
                }
            }
        }
        _ => {
            return Err(format!("Invalid table status: {}", table_status));
        }
    }

    sql.push_str("\n-- Table patch complete\n");

    Ok(sql)
}

#[tauri::command]
pub async fn generate_data_patch(
    db1_path: String,
    db2_path: String,
    table_comparisons: Vec<serde_json::Value>,
    direction: Option<String>,
    patch_type: Option<String>,
    db_manager: State<'_, Mutex<DatabaseManager>>,
) -> Result<String, String> {
    // Redirect to file-based generation for consistency and performance
    generate_data_patch_file(
        db1_path,
        db2_path,
        table_comparisons,
        direction,
        patch_type,
        db_manager,
    )
    .await
}

#[tauri::command]
pub async fn generate_data_patch_file(
    db1_path: String,
    db2_path: String,
    table_comparisons: Vec<serde_json::Value>,
    direction: Option<String>,
    patch_type: Option<String>,
    db_manager: State<'_, Mutex<DatabaseManager>>,
) -> Result<String, String> {
    let db1_password = {
        let manager = db_manager.lock().unwrap();
        manager.get_password(&db1_path).unwrap_or_default()
    };

    let db2_password = {
        let manager = db_manager.lock().unwrap();
        manager.get_password(&db2_path).unwrap_or_default()
    };

    let direction = direction.unwrap_or_else(|| "source_to_target".to_string());
    let patch_type = patch_type.unwrap_or_else(|| "all".to_string());

    // Run blocking database operations in a separate thread pool
    tokio::task::spawn_blocking(move || {
        generate_data_patch_blocking_to_file(
            &db1_path,
            &db2_path,
            &table_comparisons,
            &db1_password,
            &db2_password,
            &direction,
            &patch_type,
        )
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))?
}

fn generate_data_patch_blocking_to_file(
    db1_path: &str,
    db2_path: &str,
    table_comparisons: &[serde_json::Value],
    db1_password: &str,
    db2_password: &str,
    direction: &str,
    patch_type: &str,
) -> Result<String, String> {
    use std::fs::File;
    use std::io::Write;

    // Create a temporary file
    let temp_dir = std::env::temp_dir();
    let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S").to_string();
    let file_name = format!("plandb_patch_{}.sql", timestamp);
    let file_path = temp_dir.join(&file_name);

    let mut file =
        File::create(&file_path).map_err(|e| format!("Failed to create temp file: {}", e))?;

    let is_reverse = direction == "target_to_source";

    // Write header
    writeln!(file, "-- Data Synchronization Patch").map_err(|e| e.to_string())?;
    if is_reverse {
        writeln!(file, "-- Direction: Target → Source (Reverse)").map_err(|e| e.to_string())?;
        writeln!(file, "-- Apply to: {} (Source)", db1_path).map_err(|e| e.to_string())?;
    } else {
        writeln!(file, "-- Direction: Source → Target (Forward)").map_err(|e| e.to_string())?;
        writeln!(file, "-- Apply to: {} (Target)", db2_path).map_err(|e| e.to_string())?;
    }
    writeln!(file, "-- Generated: {} UTC", chrono::Utc::now()).map_err(|e| e.to_string())?;
    // Warnings removed as per user request
    writeln!(file, "").map_err(|e| e.to_string())?;
    writeln!(file, "BEGIN TRANSACTION;\n").map_err(|e| e.to_string())?;

    // Process tables and write directly to file
    for comparison in table_comparisons {
        let table_name = comparison["tableName"].as_str().unwrap_or("unknown");
        let key_column = comparison["keyColumn"].as_str().unwrap_or("id");

        // Get column info
        // The frontend sends the comparison data nested inside a "comparison" object
        let comparison_data = comparison.get("comparison").unwrap_or(comparison);

        let columns: Vec<String> = if let Some(cols) = comparison_data
            .get("commonColumns")
            .and_then(|v| v.as_array())
        {
            cols.iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect()
        } else {
            Vec::new()
        };

        if columns.is_empty() {
            continue;
        }

        // Handling missing rows (only if patch_type allows)
        let should_include_missing = patch_type == "all" || patch_type == "missing";
        if should_include_missing {
            if let Some(missing) = comparison_data
                .get("missingInTarget")
                .and_then(|v| v.as_array())
            {
                if !missing.is_empty() {
                    if is_reverse {
                        // Reverse: "missing in target" = "extra in source" -> DELETE from source
                        writeln!(
                            file,
                            "-- DELETE {} extra rows from {} (exist in source only)",
                            missing.len(),
                            table_name
                        )
                        .map_err(|e| e.to_string())?;
                        for row in missing {
                            // Extract key value - handle both string and non-string types
                            let key_value = match &row[key_column] {
                                serde_json::Value::String(s) => s.clone(),
                                serde_json::Value::Number(n) => n.to_string(),
                                serde_json::Value::Bool(b) => b.to_string(),
                                _ => "NULL".to_string(),
                            };
                            // DELETEs are now executable
                            writeln!(
                                file,
                                "DELETE FROM {} WHERE {} = '{}';",
                                table_name, key_column, key_value
                            )
                            .map_err(|e| e.to_string())?;
                        }
                    } else {
                        // Forward: INSERT into target
                        writeln!(
                            file,
                            "-- INSERT {} missing rows into {}",
                            missing.len(),
                            table_name
                        )
                        .map_err(|e| e.to_string())?;
                        for row in missing {
                            let insert_sql = generate_insert_statement(table_name, &columns, row)?;
                            writeln!(file, "{}", insert_sql).map_err(|e| e.to_string())?;
                        }
                    }
                    writeln!(file, "").map_err(|e| e.to_string())?;
                }
            }
        }

        // Handling extra rows (only if patch_type allows)
        let should_include_extra = patch_type == "all" || patch_type == "extra";
        if should_include_extra {
            if let Some(extra) = comparison_data
                .get("extraInTarget")
                .and_then(|v| v.as_array())
            {
                if !extra.is_empty() {
                    if is_reverse {
                        // Reverse: "extra in target" = "missing in source" -> INSERT into source
                        writeln!(
                            file,
                            "-- INSERT {} missing rows into {} (exist in target only)",
                            extra.len(),
                            table_name
                        )
                        .map_err(|e| e.to_string())?;
                        for row in extra {
                            let insert_sql = generate_insert_statement(table_name, &columns, row)?;
                            writeln!(file, "{}", insert_sql).map_err(|e| e.to_string())?;
                        }
                    } else {
                        // Forward: DELETE from target
                        writeln!(
                            file,
                            "-- DELETE {} extra rows from {}",
                            extra.len(),
                            table_name
                        )
                        .map_err(|e| e.to_string())?;
                        for row in extra {
                            // Extract key value - handle both string and non-string types
                            let key_value = match &row[key_column] {
                                serde_json::Value::String(s) => s.clone(),
                                serde_json::Value::Number(n) => n.to_string(),
                                serde_json::Value::Bool(b) => b.to_string(),
                                _ => "NULL".to_string(),
                            };
                            // DELETEs are now executable
                            writeln!(
                                file,
                                "DELETE FROM {} WHERE {} = '{}';",
                                table_name, key_column, key_value
                            )
                            .map_err(|e| e.to_string())?;
                        }
                    }
                    writeln!(file, "").map_err(|e| e.to_string())?;
                }
            }
        }

        // Handling different rows (only if patch_type allows)
        let should_include_different = patch_type == "all" || patch_type == "different";
        if should_include_different {
            if let Some(different) = comparison_data
                .get("differentRows")
                .and_then(|v| v.as_array())
            {
                if !different.is_empty() {
                    writeln!(
                        file,
                        "-- UPDATE {} different rows in {}",
                        different.len(),
                        table_name
                    )
                    .map_err(|e| e.to_string())?;
                    for diff in different {
                        let row_data = if is_reverse {
                            &diff["targetRow"] // Use target data to update source
                        } else {
                            &diff["sourceRow"] // Use source data to update target
                        };

                        // Extract key value - handle both string and non-string types
                        let key_value_json = &row_data[key_column];
                        let key_value_formatted = format_value_for_sql(key_value_json);

                        let mut set_clauses = Vec::new();
                        if let Some(diff_cols) =
                            diff.get("differentColumns").and_then(|v| v.as_array())
                        {
                            for col_val in diff_cols {
                                if let Some(col_name) = col_val.as_str() {
                                    let val = &row_data[col_name];
                                    let formatted_val = format_value_for_sql(val);
                                    set_clauses.push(format!("{} = {}", col_name, formatted_val));
                                }
                            }
                        }

                        if !set_clauses.is_empty() {
                            writeln!(
                                file,
                                "UPDATE {} SET {} WHERE {} = {};",
                                table_name,
                                set_clauses.join(", "),
                                key_column,
                                key_value_formatted
                            )
                            .map_err(|e| e.to_string())?;
                        }
                    }
                    writeln!(file, "").map_err(|e| e.to_string())?;
                }
            }
        }
    }

    writeln!(file, "COMMIT;").map_err(|e| e.to_string())?;

    // IMPORTANT: Flush the file buffer to ensure all data is written to disk
    // before reading metadata. Otherwise, metadata.len() will only reflect
    // the partial write that happened to reach the OS.
    file.flush()
        .map_err(|e| format!("Failed to flush file: {}", e))?;

    // Drop the file handle to ensure it's fully closed
    drop(file);

    // Get file size from disk (now that it's fully written)
    let metadata = std::fs::metadata(&file_path).map_err(|e| e.to_string())?;
    let file_size = metadata.len();

    // Read preview (first 5KB)
    use std::io::Read;
    let mut preview_file = File::open(&file_path).map_err(|e| e.to_string())?;
    let mut buffer = [0; 5120]; // 5KB buffer
    let bytes_read = preview_file.read(&mut buffer).unwrap_or(0);
    let preview = String::from_utf8_lossy(&buffer[..bytes_read]).to_string();

    let mut final_preview = preview;
    if file_size > 5120 {
        final_preview.push_str("\n\n... (remaining content truncated for preview) ...");
    }

    // Return JSON with file info
    let result = serde_json::json!({
        "filePath": file_path.to_string_lossy(),
        "fileSize": file_size,
        "preview": final_preview,
        "isLarge": file_size > 5 * 1024 * 1024 // Flag as large if > 5MB
    });

    Ok(result.to_string())
}

#[tauri::command]
pub async fn apply_data_patch(
    target_db_path: String,
    patch_sql: String,
    db_manager: State<'_, Mutex<DatabaseManager>>,
) -> Result<String, String> {
    const BATCH_SIZE: usize = 1000; // Commit every 1000 statements for data patches

    let db_password = {
        let manager = db_manager.lock().unwrap();
        manager.get_password(&target_db_path).unwrap_or_default()
    };

    // Open target database
    let conn = Connection::open(&target_db_path)
        .map_err(|e| format!("Failed to open target database: {}", e))?;

    // Unlock if needed
    unlock_database(&conn, &db_password)?;

    // Prepare statements - filter and clean first
    let all_statements: Vec<&str> = patch_sql.split(';').collect();
    let mut cleaned_statements: Vec<String> = Vec::new();

    for statement in all_statements.iter() {
        let trimmed = statement.trim();
        if trimmed.is_empty() {
            continue;
        }

        // Remove comment lines (lines starting with --)
        let cleaned: String = trimmed
            .lines()
            .filter(|line| !line.trim().starts_with("--"))
            .collect::<Vec<_>>()
            .join("\n");

        let cleaned = cleaned.trim();
        if !cleaned.is_empty() {
            cleaned_statements.push(cleaned.to_string());
        }
    }

    let total_statements = cleaned_statements.len();
    let mut executed = 0;
    let mut in_transaction = false;

    // Execute statements in batches
    for (idx, statement) in cleaned_statements.iter().enumerate() {
        let stmt_upper = statement.to_uppercase();

        // Handle transaction commands
        if stmt_upper.starts_with("BEGIN") {
            if !in_transaction {
                conn.execute(statement, [])
                    .map_err(|e| format!("Failed to begin transaction: {}", e))?;
                in_transaction = true;
            }
            continue;
        }

        if stmt_upper.starts_with("COMMIT") {
            if in_transaction {
                conn.execute(statement, [])
                    .map_err(|e| format!("Failed to commit transaction: {}", e))?;
                in_transaction = false;
            }
            continue;
        }

        // Start transaction if not already in one
        if !in_transaction {
            conn.execute("BEGIN IMMEDIATE", [])
                .map_err(|e| format!("Failed to begin transaction: {}", e))?;
            in_transaction = true;
        }

        // Execute the statement
        conn.execute(statement, []).map_err(|e| {
            // Try to rollback on error
            if in_transaction {
                let _ = conn.execute("ROLLBACK", []);
            }
            format!(
                "Error at statement {}/{}: {}\nStatement: {}",
                idx + 1,
                total_statements,
                e,
                statement
            )
        })?;

        executed += 1;

        // Commit and start new transaction every BATCH_SIZE statements
        // This prevents long-running transactions that lock the database
        if executed % BATCH_SIZE == 0 && idx < cleaned_statements.len() - 1 {
            conn.execute("COMMIT", [])
                .map_err(|e| format!("Failed to commit batch: {}", e))?;

            in_transaction = false;

            // Yield to system to prevent UI freeze
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

            conn.execute("BEGIN IMMEDIATE", [])
                .map_err(|e| format!("Failed to begin new batch: {}", e))?;

            in_transaction = true;
        }
    }

    // Final commit if still in transaction
    if in_transaction {
        conn.execute("COMMIT", [])
            .map_err(|e| format!("Failed to final commit: {}", e))?;
    }

    Ok(format!(
        "Data patch applied successfully. Executed {} statements.",
        executed
    ))
}

// Helper to format values for SQL
fn format_value_for_sql(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::Null => "NULL".to_string(),
        serde_json::Value::Number(n) => n.to_string(),
        serde_json::Value::String(s) => format!("'{}'", s.replace("'", "''")), // Escape single quotes
        serde_json::Value::Bool(b) => (if *b { "1" } else { "0" }).to_string(),
        _ => format!("'{}'", value.to_string().replace("'", "''")),
    }
}

// Helper to generate INSERT statement
fn generate_insert_statement(
    table_name: &str,
    columns: &[String],
    row: &serde_json::Value,
) -> Result<String, String> {
    let mut values = Vec::new();
    for col in columns {
        let val = row.get(col).unwrap_or(&serde_json::Value::Null);
        values.push(format_value_for_sql(val));
    }

    Ok(format!(
        "INSERT INTO {} ({}) VALUES ({});",
        table_name,
        columns.join(", "),
        values.join(", ")
    ))
}

#[tauri::command]
pub async fn get_license_status(
    license_manager: State<'_, LicenseManagerState>,
) -> Result<LicenseStatus, String> {
    let manager = license_manager.lock().unwrap();
    manager.get_license_status()
}

#[tauri::command]
pub async fn activate_license(
    email: String,
    license_key: String,
    _license_manager: State<'_, LicenseManagerState>,
) -> Result<LicenseStatus, String> {
    // Validate first
    if email.is_empty() || license_key.is_empty() {
        return Err("Email and license key are required".to_string());
    }

    if !email.contains('@') {
        return Err("Invalid email address".to_string());
    }

    // Validate format (inline, no borrowing issues)
    if license_key.len() != 19 {
        return Err("Invalid license key format".to_string());
    }
    let parts: Vec<&str> = license_key.split('-').collect();
    if parts.len() != 4
        || !parts
            .iter()
            .all(|p| p.len() == 4 && p.chars().all(|c| c.is_alphanumeric()))
    {
        return Err("Invalid license key format".to_string());
    }

    // Verify with server (async, no lock needed)
    let license_info = verify_license_with_server(&email, &license_key).await?;

    // Save license (lock only for file write)
    save_license_to_file(&license_info)?;

    // Return status
    Ok(LicenseStatus {
        is_valid: true,
        license_type: license_info.license_type.clone(),
        days_remaining: license_info
            .expiry_date
            .map(|exp| (exp - Utc::now()).num_days()),
        message: "License activated successfully".to_string(),
    })
}

#[tauri::command]
pub async fn deactivate_license(
    _license_manager: State<'_, LicenseManagerState>,
) -> Result<String, String> {
    // Just delete the license file
    let license_file = get_license_file_path()?;
    if license_file.exists() {
        fs::remove_file(&license_file).map_err(|e| format!("Failed to remove license: {}", e))?;
    }
    Ok("License deactivated successfully".to_string())
}

#[tauri::command]
pub async fn check_trial_status(
    license_manager: State<'_, LicenseManagerState>,
) -> Result<LicenseStatus, String> {
    let manager = license_manager.lock().unwrap();
    manager.get_license_status()
}

// Helper functions (add these at the end of commands.rs)

async fn verify_license_with_server(
    email: &str,
    license_key: &str,
) -> Result<crate::license::LicenseInfo, String> {
    let client = reqwest::Client::new();
    let machine_id = get_machine_id_inline();

    // let server_url = "http://localhost:3000/api/verify";
    //let server_url = "https://plandbdiff-licence.vercel.app/api/verify";
    let server_url = "https://plandbdiff-license-02-1xyxdl83k-manikandans-projects-be37ef3a.vercel.app/api/verify";

    let response = client
        .post(server_url)
        .json(&serde_json::json!({
            "email": email,
            "license_key": license_key,
            "machine_id": machine_id,
        }))
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await
        .map_err(|e| {
            format!(
                "License verification failed: {}. Check internet connection.",
                e
            )
        })?;

    if !response.status().is_success() {
        return Err("Invalid license key or email".to_string());
    }

    let data: serde_json::Value = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse server response: {}", e))?;

    let license_type = match data["license_type"].as_str() {
        Some("monthly") => crate::license::LicenseType::Monthly,
        Some("yearly") => crate::license::LicenseType::Yearly,
        Some("lifetime") => crate::license::LicenseType::Lifetime,
        _ => return Err("Invalid license type from server".to_string()),
    };

    let expiry_date = if license_type == crate::license::LicenseType::Lifetime {
        None
    } else {
        Some(
            DateTime::parse_from_rfc3339(
                data["expiry_date"].as_str().ok_or("Missing expiry date")?,
            )
            .map_err(|e| format!("Invalid expiry date: {}", e))?
            .with_timezone(&Utc),
        )
    };

    Ok(crate::license::LicenseInfo {
        email: email.to_string(),
        license_key: license_key.to_string(),
        license_type,
        activation_date: Utc::now(),
        expiry_date,
        last_validation: Utc::now(),
        machine_id,
    })
}

fn save_license_to_file(license: &crate::license::LicenseInfo) -> Result<(), String> {
    let license_file = get_license_file_path()?;
    let json = serde_json::to_string_pretty(license)
        .map_err(|e| format!("Failed to serialize license: {}", e))?;

    fs::write(&license_file, json).map_err(|e| format!("Failed to save license: {}", e))?;

    Ok(())
}

fn get_license_file_path() -> Result<PathBuf, String> {
    let app_data_dir = get_app_data_dir_inline()?;
    Ok(app_data_dir.join("license.json"))
}

fn get_app_data_dir_inline() -> Result<PathBuf, String> {
    let app_name = "SQLCipherTool";

    #[cfg(target_os = "windows")]
    {
        let appdata =
            std::env::var("APPDATA").map_err(|_| "APPDATA environment variable not found")?;
        Ok(PathBuf::from(appdata).join(app_name))
    }

    #[cfg(target_os = "macos")]
    {
        let home = std::env::var("HOME").map_err(|_| "HOME environment variable not found")?;
        Ok(PathBuf::from(home)
            .join("Library")
            .join("Application Support")
            .join(app_name))
    }

    #[cfg(target_os = "linux")]
    {
        let home = std::env::var("HOME").map_err(|_| "HOME environment variable not found")?;
        Ok(PathBuf::from(home).join(".config").join(app_name))
    }
}

fn get_machine_id_inline() -> String {
    use sha2::{Digest, Sha256};

    let mut hasher = Sha256::new();

    if let Ok(hostname) = hostname::get() {
        hasher.update(hostname.to_string_lossy().as_bytes());
    }

    if let Ok(Some(ma)) = mac_address::get_mac_address() {
        hasher.update(ma.to_string().as_bytes());
    }

    hasher.update(std::env::consts::OS.as_bytes());

    let result = hasher.finalize();
    format!("{:x}", result)
}

#[tauri::command]
pub async fn migrate_to_sqlcipher(
    source_path: String,
    password: String,
    settings: MigrationSettings,
) -> Result<MigrationResult, String> {
    // Validate source file exists
    if !Path::new(&source_path).exists() {
        return Err("Source database file not found".to_string());
    }

    // Generate output filename
    let source = Path::new(&source_path);
    let file_stem = source
        .file_stem()
        .and_then(|s| s.to_str())
        .ok_or("Invalid source filename")?;

    let timestamp = Local::now().format("%Y%m%d_%H%M%S");
    let output_filename = format!("{}_encrypted_{}.db", file_stem, timestamp);

    let output_path = source
        .parent()
        .ok_or("Invalid source path")?
        .join(&output_filename)
        .to_str()
        .ok_or("Invalid output path")?
        .to_string();

    // Open source database (unencrypted SQLite)
    let source_conn = Connection::open(&source_path)
        .map_err(|e| format!("Failed to open source database: {}", e))?;

    // Get list of tables - FIXED
    let tables: Vec<String> = {
        let mut stmt = source_conn
            .prepare(
                "SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%'",
            )
            .map_err(|e| format!("Failed to query tables: {}", e))?;

        let rows = stmt
            .query_map([], |row| row.get(0))
            .map_err(|e| format!("Failed to read tables: {}", e))?;

        let mut result = Vec::new();
        for row in rows {
            result.push(row.map_err(|e| format!("Failed to collect tables: {}", e))?);
        }
        result
    };

    // Get table schemas
    let mut table_schemas = Vec::new();
    for table_name in &tables {
        let schema: String = {
            let mut schema_stmt = source_conn
                .prepare("SELECT sql FROM sqlite_master WHERE type='table' AND name=?1")
                .map_err(|e| format!("Failed to prepare schema query: {}", e))?;

            schema_stmt
                .query_row([table_name], |row| row.get(0))
                .map_err(|e| format!("Failed to get schema for table {}: {}", table_name, e))?
        };

        table_schemas.push((table_name.clone(), schema));
    }

    // Get indexes - FIXED
    let indexes: Vec<String> = {
        let mut idx_stmt = source_conn
            .prepare("SELECT sql FROM sqlite_master WHERE type='index' AND sql IS NOT NULL")
            .map_err(|e| format!("Failed to query indexes: {}", e))?;

        let rows = idx_stmt
            .query_map([], |row| row.get(0))
            .map_err(|e| format!("Failed to read indexes: {}", e))?;

        let mut result = Vec::new();
        for row in rows {
            result.push(row.map_err(|e| format!("Failed to collect indexes: {}", e))?);
        }
        result
    };

    // Get triggers - FIXED
    let triggers: Vec<String> = {
        let mut trigger_stmt = source_conn
            .prepare("SELECT sql FROM sqlite_master WHERE type='trigger'")
            .map_err(|e| format!("Failed to query triggers: {}", e))?;

        let rows = trigger_stmt
            .query_map([], |row| row.get(0))
            .map_err(|e| format!("Failed to read triggers: {}", e))?;

        let mut result = Vec::new();
        for row in rows {
            result.push(row.map_err(|e| format!("Failed to collect triggers: {}", e))?);
        }
        result
    };

    // Get views - FIXED
    let views: Vec<String> = {
        let mut view_stmt = source_conn
            .prepare("SELECT sql FROM sqlite_master WHERE type='view'")
            .map_err(|e| format!("Failed to query views: {}", e))?;

        let rows = view_stmt
            .query_map([], |row| row.get(0))
            .map_err(|e| format!("Failed to read views: {}", e))?;

        let mut result = Vec::new();
        for row in rows {
            result.push(row.map_err(|e| format!("Failed to collect views: {}", e))?);
        }
        result
    };

    // Create encrypted database with SQLCipher
    let mut dest_conn = Connection::open(&output_path)
        .map_err(|e| format!("Failed to create destination database: {}", e))?;

    // Set encryption key and settings
    dest_conn
        .pragma_update(None, "key", &password)
        .map_err(|e| format!("Failed to set encryption key: {}", e))?;

    apply_sqlcipher_settings(&mut dest_conn, &settings)?;

    // Begin transaction
    dest_conn
        .execute("BEGIN TRANSACTION", [])
        .map_err(|e| format!("Failed to begin transaction: {}", e))?;

    // Create tables with fixed collations
    for (table_name, schema) in &table_schemas {
        // Replace unsupported collations with SQLCipher-compatible ones
        // SQLCipher only supports: BINARY, NOCASE, RTRIM
        let fixed_schema = schema
            .replace("COLLATE UTF16", "COLLATE BINARY")
            .replace("COLLATE utf16", "COLLATE BINARY")
            .replace("COLLATE UTF8", "COLLATE BINARY")
            .replace("COLLATE utf8", "COLLATE BINARY");

        dest_conn
            .execute(&fixed_schema, [])
            .map_err(|e| format!("Failed to create table {}: {}", table_name, e))?;
    }

    // Copy data for each table
    for table_name in &tables {
        let column_count: usize = {
            let col_stmt = source_conn
                .prepare(&format!("SELECT * FROM {} LIMIT 0", table_name))
                .map_err(|e| format!("Failed to get columns for {}: {}", table_name, e))?;
            col_stmt.column_count()
        };

        // Read all data - FIXED
        let rows_data: Vec<Vec<rusqlite::types::Value>> = {
            let mut data_stmt = source_conn
                .prepare(&format!("SELECT * FROM {}", table_name))
                .map_err(|e| format!("Failed to prepare data query for {}: {}", table_name, e))?;

            let rows = data_stmt
                .query_map([], |row| {
                    let mut values = Vec::new();
                    for i in 0..column_count {
                        values.push(row.get::<_, rusqlite::types::Value>(i)?);
                    }
                    Ok(values)
                })
                .map_err(|e| format!("Failed to query data from {}: {}", table_name, e))?;

            let mut result = Vec::new();
            for row in rows {
                result.push(
                    row.map_err(|e| format!("Failed to read rows from {}: {}", table_name, e))?,
                );
            }
            result
        };

        // Insert data
        let placeholders = (0..column_count)
            .map(|_| "?")
            .collect::<Vec<_>>()
            .join(", ");
        let insert_sql = format!("INSERT INTO {} VALUES ({})", table_name, placeholders);

        for row_values in rows_data {
            let params: Vec<&dyn rusqlite::ToSql> = row_values
                .iter()
                .map(|v| v as &dyn rusqlite::ToSql)
                .collect();

            dest_conn
                .execute(&insert_sql, params.as_slice())
                .map_err(|e| format!("Failed to insert data into {}: {}", table_name, e))?;
        }
    }

    // Create indexes
    for index_sql in indexes {
        dest_conn
            .execute(&index_sql, [])
            .map_err(|e| format!("Failed to create index: {}", e))?;
    }

    // Create triggers
    for trigger_sql in triggers {
        dest_conn
            .execute(&trigger_sql, [])
            .map_err(|e| format!("Failed to create trigger: {}", e))?;
    }

    // Create views
    for view_sql in views {
        dest_conn
            .execute(&view_sql, [])
            .map_err(|e| format!("Failed to create view: {}", e))?;
    }

    // Commit transaction
    dest_conn
        .execute("COMMIT", [])
        .map_err(|e| format!("Failed to commit transaction: {}", e))?;

    // Close connections
    drop(dest_conn);
    drop(source_conn);

    Ok(MigrationResult {
        output_path: output_path.clone(),
        message: format!("Successfully migrated to SQLCipher: {}", output_filename),
        success: true,
    })
}

#[tauri::command]
pub async fn check_database_type(db_path: String) -> Result<String, String> {
    use std::io::Read;

    // Check if database is encrypted (SQLCipher) or plain SQLite
    let mut file =
        std::fs::File::open(&db_path).map_err(|e| format!("Failed to open file: {}", e))?;

    let mut buffer = [0u8; 16];
    file.read_exact(&mut buffer)
        .map_err(|e| format!("Failed to read file header: {}", e))?;

    // SQLite files start with "SQLite format 3\0"
    let sqlite_magic = b"SQLite format 3\0";

    if buffer.starts_with(sqlite_magic) {
        Ok("sqlite".to_string())
    } else {
        // If it doesn't start with SQLite magic, it's likely encrypted
        Ok("sqlcipher".to_string())
    }
}

#[tauri::command]
pub async fn rekey_sqlcipher_database(
    db_path: String,
    old_password: String,
    new_password: String,
    settings: MigrationSettings,
) -> Result<MigrationResult, String> {
    use std::path::Path;

    // Validate file exists
    if !Path::new(&db_path).exists() {
        return Err("Database file not found".to_string());
    }

    // Open database with old password
    let mut conn =
        Connection::open(&db_path).map_err(|e| format!("Failed to open database: {}", e))?;

    // Set old encryption key
    conn.pragma_update(None, "key", &old_password)
        .map_err(|e| format!("Failed to unlock database with old password: {}", e))?;

    // Verify old password works by trying to read
    conn.execute("SELECT count(*) FROM sqlite_master", [])
        .map_err(|_| "Old password is incorrect or database is corrupted".to_string())?;

    // Apply new settings before rekeying
    apply_sqlcipher_settings(&mut conn, &settings)?;

    // Rekey with new password
    conn.pragma_update(None, "rekey", &new_password)
        .map_err(|e| format!("Failed to rekey database: {}", e))?;

    // Close connection
    drop(conn);

    // Verify new password works
    let mut verify_conn =
        Connection::open(&db_path).map_err(|e| format!("Failed to reopen database: {}", e))?;

    verify_conn
        .pragma_update(None, "key", &new_password)
        .map_err(|e| format!("Failed to verify new password: {}", e))?;

    apply_sqlcipher_settings(&mut verify_conn, &settings)?;

    verify_conn
        .execute("SELECT count(*) FROM sqlite_master", [])
        .map_err(|_| "Rekey verification failed".to_string())?;

    drop(verify_conn);

    Ok(MigrationResult {
        output_path: db_path.clone(),
        message: "Database successfully rekeyed with new password".to_string(),
        success: true,
    })
}

// Helper function - NOT a tauri command, so no #[tauri::command] attribute
fn apply_sqlcipher_settings(
    conn: &mut Connection,
    settings: &MigrationSettings,
) -> Result<(), String> {
    // Set cipher page size
    conn.pragma_update(None, "cipher_page_size", &settings.page_size)
        .map_err(|e| format!("Failed to set page size: {}", e))?;

    // Set KDF iterations
    conn.pragma_update(None, "kdf_iter", &settings.kdf_iterations)
        .map_err(|e| format!("Failed to set KDF iterations: {}", e))?;

    // Set HMAC algorithm (default: SHA512 for SQLCipher 4)
    let hmac_value = match settings.hmac_algorithm.as_str() {
        "HMAC_SHA1" => "HMAC_SHA1",
        "HMAC_SHA256" => "HMAC_SHA256",
        "HMAC_SHA512" => "HMAC_SHA512",
        _ => "HMAC_SHA512",
    };
    conn.pragma_update(None, "cipher_hmac_algorithm", hmac_value)
        .map_err(|e| format!("Failed to set HMAC algorithm: {}", e))?;

    // Set KDF algorithm (default: SHA512 for SQLCipher 4)
    let kdf_value = match settings.kdf_algorithm.as_str() {
        "PBKDF2_HMAC_SHA1" => "PBKDF2_HMAC_SHA1",
        "PBKDF2_HMAC_SHA256" => "PBKDF2_HMAC_SHA256",
        "PBKDF2_HMAC_SHA512" => "PBKDF2_HMAC_SHA512",
        _ => "PBKDF2_HMAC_SHA512",
    };
    conn.pragma_update(None, "cipher_kdf_algorithm", kdf_value)
        .map_err(|e| format!("Failed to set KDF algorithm: {}", e))?;

    Ok(())
}
