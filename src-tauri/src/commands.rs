

use crate::database::DatabaseManager;
use crate::models::*;
use std::sync::Mutex;
use std::time::Duration;
use tauri::State;
use tokio::time::timeout;
use rusqlite::Connection;
use std::path::Path;
use chrono::Local;
use std::io::Read;


use crate::license::{LicenseManager, LicenseStatus};
use std::fs;
use std::path::PathBuf;
use chrono::{DateTime, Utc};  // ← Add DateTime here
// ADD THIS TYPE ALIAS AFTER YOUR IMPORTS
type LicenseManagerState = Mutex<LicenseManager>;
type DbManager = Mutex<DatabaseManager>;



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
        },
        Err(e) => {
            println!("❌ Failed to connect: {}", e);
            Err(format!("Connection failed: {}", e))
        }
    }
}

#[tauri::command]
pub async fn get_table_info(db_path: String, table_name: String) -> Result<TableInfo, String> {
    use rusqlite::Connection;
    
    let conn = Connection::open(&db_path)
        .map_err(|e| format!("Failed to open database: {}", e))?;
    
    let mut stmt = conn.prepare(&format!("PRAGMA table_info({})", table_name))
        .map_err(|e| format!("Failed to get table info: {}", e))?;
    
    let mut columns = Vec::new();
    let rows = stmt.query_map([], |row| {
        Ok(ColumnInfo {
            name: row.get(1)?,
            data_type: row.get(2)?,
            is_nullable: row.get::<_, i32>(3)? == 0,
            default_value: row.get(4).ok(),
            is_primary_key: row.get::<_, i32>(5)? == 1,
        })
    }).map_err(|e| format!("Query failed: {}", e))?;
    
    for row in rows {
        columns.push(row.map_err(|e| format!("Row error: {}", e))?);
    }
    
    let row_count: i64 = conn.query_row(
        &format!("SELECT COUNT(*) FROM {}", table_name),
        [],
        |row| row.get(0)
    ).unwrap_or(0);
    
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
        },
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
    offset: Option<i64>,  // ← NEW PARAMETER
    manager: State<'_, DbManager>,
) -> Result<TableData, String> {
    // Add timeout for large queries (60 seconds)
    let timeout_duration = Duration::from_secs(60);
    
    let result = timeout(timeout_duration, async {
        let db_manager = manager.lock().unwrap();
        
        // Log the request with more details
        println!("Fetching table '{}' from '{}' with limit: {:?}, offset: {:?}", 
                table_name, db_path, limit, offset);
        
        db_manager.get_table_data(&db_path, &table_name, limit, offset)  // ← Pass offset
    }).await;
    
    match result {
        Ok(data_result) => {
            match data_result {
                Ok(data) => {
                    println!("Successfully retrieved {} rows from table '{}' in '{}'", 
                            data.rows.len(), table_name, db_path);
                    Ok(data)
                },
                Err(e) => {
                    println!("Database error for table '{}': {}", table_name, e);
                    Err(format!("Database error: {}", e))
                }
            }
        },
        Err(_) => {
            println!("Timeout occurred while fetching table '{}' (limit: {:?}, offset: {:?})", 
                    table_name, limit, offset);
            Err(format!("Query timeout - table '{}' took too long to fetch. Try using a smaller row limit.", 
                       table_name))
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
            println!("Schema comparison completed between {} and {}", db1_path, db2_path);
            Ok(comparison)
        },
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
            println!("Data comparison completed: {} total rows", result.total_rows_db1);
            Ok(result)
        },
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
    db_manager: State<'_, Mutex<DatabaseManager>>,
) -> Result<String, String> {
    let manager = db_manager.lock().unwrap();
    
    let comparison = manager.compare_schemas(&db1_path, &db2_path)
        .map_err(|e| e.to_string())?;

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
    sql.push_str(&format!("-- Source: {}\n", comparison.database1));
    sql.push_str(&format!("-- Target: {}\n", comparison.database2));
    sql.push_str(&format!("-- Generated: {}\n\n", chrono::Utc::now()));
    sql.push_str("-- NOTE: PRAGMA statements require the transaction to be committed first.\n");
    sql.push_str("-- If running in DB Browser, close any open transactions before executing.\n\n");

    // Track if we need PRAGMA statements (table recreation)
    let has_table_recreation = comparison.modified_tables.iter()
        .any(|m| !m.removed_columns.is_empty() || !m.added_columns.is_empty() || !m.modified_columns.is_empty());

    // Start transaction only for simple operations (no PRAGMA needed)
    if !has_table_recreation {
        sql.push_str("BEGIN TRANSACTION;\n\n");
    }

    // Create tables that exist in SOURCE but not in TARGET
    // Industry standard: Make target match source
    for table in &comparison.removed_tables {
        match get_create_table_sql(&source_conn, table) {
            Ok(create_sql) => {
                sql.push_str("-- Create table from source\n");
                sql.push_str(&format!("{};\n\n", create_sql));
            }
            Err(e) => {
                sql.push_str(&format!("-- ERROR: Could not get schema for {}: {}\n\n", table, e));
            }
        }
    }

    // Drop tables that exist in TARGET but not in SOURCE
    // Industry standard: Make target match source
    for table in &comparison.added_tables {
        sql.push_str("-- Drop table not in source\n");
        sql.push_str(&format!("DROP TABLE IF EXISTS `{}`;\n\n", table));
    }
    
    // Modify existing tables
    for modified in &comparison.modified_tables {
        // added_columns = columns in TARGET but not in SOURCE → need to DROP (requires recreation)
        // removed_columns = columns in SOURCE but not in TARGET → need to ADD
        // modified_columns = columns with different types/constraints → requires recreation

        let needs_recreation = !modified.added_columns.is_empty()
            || !modified.modified_columns.is_empty();

        if needs_recreation {
            // Generate table recreation SQL
            sql.push_str(&format!("-- Recreate table: {} (columns removed/modified)\n", modified.table_name));

            match generate_table_recreation_sql(
                &source_conn,
                &target_conn,
                &modified.table_name,
                &modified
            ) {
                Ok(recreation_sql) => {
                    sql.push_str(&recreation_sql);
                }
                Err(e) => {
                    sql.push_str(&format!("-- ERROR: Could not generate recreation SQL for {}: {}\n",
                        modified.table_name, e));
                    sql.push_str(&format!("-- Manual recreation required for:\n"));
                    if !modified.added_columns.is_empty() {
                        sql.push_str(&format!("--   Columns to drop: {}\n",
                            modified.added_columns.iter().map(|c| c.name.as_str()).collect::<Vec<_>>().join(", ")));
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
            // Only removed_columns (in source, not in target) - simple ALTER TABLE ADD
            sql.push_str(&format!("-- Modify table: {} (add columns from source)\n", modified.table_name));

            // Get column definitions from source
            for col_name in &modified.removed_columns {
                // Get the column info from source database
                match get_column_info(&source_conn, &modified.table_name, col_name) {
                    Ok(col) => {
                        let nullable = if col.is_nullable { "" } else { " NOT NULL" };
                        let default = match &col.default_value {
                            Some(def) => format!(" DEFAULT {}", def),
                            None => String::new(),
                        };

                        sql.push_str(&format!(
                            "ALTER TABLE `{}` ADD COLUMN `{}` {}{}{};\n",
                            modified.table_name,
                            col.name,
                            col.data_type,
                            nullable,
                            default
                        ));
                    }
                    Err(e) => {
                        sql.push_str(&format!("-- ERROR: Could not get column info for {}: {}\n", col_name, e));
                    }
                }
            }
            sql.push_str("\n");
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

    // Start initial transaction
    conn.execute("BEGIN IMMEDIATE", [])
        .map_err(|e| format!("Failed to begin transaction: {}", e))?;

    // Execute statements in batches
    for (idx, statement) in cleaned_statements.iter().enumerate() {
        // Skip BEGIN/COMMIT from the patch itself - we manage transactions
        let stmt_upper = statement.to_uppercase();
        if stmt_upper.starts_with("BEGIN") || stmt_upper.starts_with("COMMIT") {
            continue;
        }

        // Execute the statement
        conn.execute(statement, [])
            .map_err(|e| {
                // Try to rollback on error
                let _ = conn.execute("ROLLBACK", []);
                format!("Error at statement {}/{}: {}\nStatement: {}",
                    idx + 1, total_statements, e, statement)
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

    Ok(format!("Schema patch applied successfully. Executed {} statements.", executed))
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
                if conn.query_row(
                    "SELECT COUNT(*) FROM sqlite_master WHERE type='table';", 
                    [], 
                    |_| Ok(())
                ).is_ok() {
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
        |_| Ok(())
    ).map_err(|e| format!("Failed to read database: {}", e))?;
    
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
    let common_columns: Vec<String> = source_columns.iter()
        .filter(|col| target_columns.contains(col))
        .cloned()
        .collect();
    
    if common_columns.is_empty() {
        return Err("No common columns found - table would lose all data".to_string());
    }
    
    let common_cols_str = common_columns.iter()
        .map(|c| format!("`{}`", c))
        .collect::<Vec<_>>()
        .join(", ");
    
    let mut sql = String::new();
    
    // Step 1: Disable foreign keys (must be outside transaction)
    sql.push_str("PRAGMA foreign_keys=off;\n\n");

    // Step 2: Start a transaction for this table recreation
    sql.push_str("BEGIN IMMEDIATE;\n\n");
    
    // Step 3: Rename old table (current target)
    sql.push_str(&format!("ALTER TABLE `{}` RENAME TO `{}_old`;\n\n", table_name, table_name));

    // Step 4: Create new table with source schema (make target match source)
    sql.push_str(&format!("{};\n\n", source_schema));
    
    // Step 5: Copy data from old to new table
    sql.push_str(&format!(
        "INSERT INTO `{}` ({})\nSELECT {}\nFROM `{}_old`;\n\n",
        table_name,
        common_cols_str,
        common_cols_str,
        table_name
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
        sql.push_str(&format!("--   Added (from source): {}\n", diff.removed_columns.join(", ")));
    }
    if !diff.added_columns.is_empty() {
        let dropped: Vec<String> = diff.added_columns.iter().map(|c| c.name.clone()).collect();
        sql.push_str(&format!("--   Dropped (not in source): {}\n", dropped.join(", ")));
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

// Get column names from a table
fn get_table_column_names(conn: &Connection, table_name: &str) -> Result<Vec<String>, String> {
    let mut stmt = conn.prepare(&format!("PRAGMA table_info(`{}`)", table_name))
        .map_err(|e| format!("Failed to get table info: {}", e))?;
    
    let columns: Vec<String> = stmt.query_map([], |row| {
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


#[tauri::command]
pub async fn generate_data_patch(
    db1_path: String,
    db2_path: String,
    table_comparisons: Vec<serde_json::Value>, // From frontend comparison results
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

    // Run blocking database operations in a separate thread pool
    tokio::task::spawn_blocking(move || {
        generate_data_patch_blocking(
            &db1_path,
            &db2_path,
            &table_comparisons,
            &db1_password,
            &db2_password
        )
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))?
}

// Blocking version that does the actual work
fn generate_data_patch_blocking(
    db1_path: &str,
    db2_path: &str,
    table_comparisons: &[serde_json::Value],
    db1_password: &str,
    db2_password: &str,
) -> Result<String, String> {
    // Open both databases
    let source_conn = Connection::open(db1_path)
        .map_err(|e| format!("Failed to open source database: {}", e))?;

    let target_conn = Connection::open(db2_path)
        .map_err(|e| format!("Failed to open target database: {}", e))?;

    // Unlock databases if needed
    unlock_database(&source_conn, db1_password)?;
    unlock_database(&target_conn, db2_password)?;

    // Pre-calculate approximate size to avoid reallocations
    // Estimate: ~200 bytes per row on average
    let mut estimated_rows = 0;
    for table_json in table_comparisons.iter() {
        if let Some(comparison) = table_json.get("comparison") {
            if let Some(missing) = comparison.get("missingInTarget").and_then(|v| v.as_array()) {
                estimated_rows += missing.len();
            }
            if let Some(different) = comparison.get("differentRows").and_then(|v| v.as_array()) {
                estimated_rows += different.len();
            }
            if let Some(extra) = comparison.get("extraInTarget").and_then(|v| v.as_array()) {
                estimated_rows += extra.len();
            }
        }
    }
    let estimated_size = estimated_rows * 200 + 1024; // 200 bytes per row + header

    // Generate SQL patch with pre-allocated capacity
    let mut sql = String::with_capacity(estimated_size);
    sql.push_str("-- Data Synchronization Patch\n");
    sql.push_str(&format!("-- Source: {}\n", db1_path));
    sql.push_str(&format!("-- Target: {}\n", db2_path));
    sql.push_str(&format!("-- Generated: {}\n\n", chrono::Utc::now()));
    
    sql.push_str("-- IMPORTANT: Review this script before execution!\n");
    sql.push_str("-- DELETE statements are commented out by default for safety.\n\n");
    
    sql.push_str("BEGIN TRANSACTION;\n\n");

    // Process each table from comparison results
    for table_json in table_comparisons {
        let table_name = table_json.get("tableName")
            .and_then(|v| v.as_str())
            .ok_or("Missing table name")?;

        let key_column = table_json.get("keyColumn")
            .and_then(|v| v.as_str())
            .ok_or("Missing key column")?;

        let comparison = table_json.get("comparison")
            .ok_or("Missing comparison data")?;

        sql.push_str(&format!("-- ====================================\n"));
        sql.push_str(&format!("-- Table: {}\n", table_name));
        sql.push_str(&format!("-- Key Column: {}\n", key_column));
        sql.push_str(&format!("-- ====================================\n\n"));

        // Get columns info
        let columns = get_column_names(&target_conn, table_name)?;

        // 1. INSERT missing rows (exist in source, not in target)
        if let Some(missing) = comparison.get("missingInTarget").and_then(|v| v.as_array()) {
            if !missing.is_empty() {
                sql.push_str(&format!("-- INSERT {} missing rows into {}\n", missing.len(), table_name));

                for (idx, row) in missing.iter().enumerate() {
                    let insert_sql = generate_insert_statement(
                        table_name,
                        &columns,
                        row
                    )?;
                    sql.push_str(&insert_sql);
                    sql.push_str("\n");

                    // Yield every 1000 rows to allow other work
                    if idx % 1000 == 0 && idx > 0 {
                        std::thread::yield_now();
                    }
                }
                sql.push_str("\n");
            }
        }

        // 2. UPDATE different rows
        if let Some(different) = comparison.get("differentRows").and_then(|v| v.as_array()) {
            if !different.is_empty() {
                sql.push_str(&format!("-- UPDATE {} different rows in {}\n", different.len(), table_name));

                for (idx, diff) in different.iter().enumerate() {
                    let source_row = diff.get("sourceRow")
                        .ok_or("Missing source row")?;

                    let different_cols = diff.get("differentColumns")
                        .and_then(|v| v.as_array())
                        .ok_or("Missing different columns")?;

                    let update_sql = generate_update_statement(
                        table_name,
                        key_column,
                        source_row,
                        different_cols
                    )?;
                    sql.push_str(&update_sql);
                    sql.push_str("\n");

                    // Yield every 1000 rows to allow other work
                    if idx % 1000 == 0 && idx > 0 {
                        std::thread::yield_now();
                    }
                }
                sql.push_str("\n");
            }
        }

        // 3. DELETE extra rows (exist in target, not in source)
        // Industry standard: Make target match source by removing extra rows
        if let Some(extra) = comparison.get("extraInTarget").and_then(|v| v.as_array()) {
            if !extra.is_empty() {
                sql.push_str(&format!("-- DELETE {} extra rows from {}\n",
                    extra.len(), table_name));

                for (idx, row) in extra.iter().enumerate() {
                    let key_value = row.get(key_column)
                        .ok_or_else(|| format!("Missing key column {} in row", key_column))?;

                    let delete_sql = format!(
                        "DELETE FROM `{}` WHERE `{}` = {};\n",
                        table_name,
                        key_column,
                        format_value_for_sql(key_value)
                    );
                    sql.push_str(&delete_sql);

                    // Yield every 1000 rows to allow other work
                    if idx % 1000 == 0 && idx > 0 {
                        std::thread::yield_now();
                    }
                }
                sql.push_str("\n");
            }
        }
    }
    
    sql.push_str("COMMIT;\n\n");
    sql.push_str("-- End of data synchronization patch\n");

    Ok(sql)
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
        conn.execute(statement, [])
            .map_err(|e| {
                // Try to rollback on error
                if in_transaction {
                    let _ = conn.execute("ROLLBACK", []);
                }
                format!("Error at statement {}/{}: {}\nStatement: {}",
                    idx + 1, total_statements, e, statement)
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

    Ok(format!("Data patch applied successfully. Executed {} statements.", executed))
}

// Helper function to generate INSERT statement
fn generate_insert_statement(
    table_name: &str,
    columns: &[String],
    row: &serde_json::Value,
) -> Result<String, String> {
    let col_list = columns.iter()
        .map(|c| format!("`{}`", c))
        .collect::<Vec<_>>()
        .join(", ");
    
    let values = columns.iter()
        .map(|col| {
            let value = row.get(col)
                .unwrap_or(&serde_json::Value::Null);
            format_value_for_sql(value)
        })
        .collect::<Vec<_>>()
        .join(", ");
    
    Ok(format!(
        "INSERT INTO `{}` ({}) VALUES ({});",
        table_name, col_list, values
    ))
}

// Helper function to generate UPDATE statement
fn generate_update_statement(
    table_name: &str,
    key_column: &str,
    source_row: &serde_json::Value,
    different_columns: &[serde_json::Value],
) -> Result<String, String> {
    let key_value = source_row.get(key_column)
        .ok_or_else(|| format!("Missing key column {}", key_column))?;
    
    let set_clauses = different_columns.iter()
        .filter_map(|col| col.as_str())
        .map(|col| {
            let value = source_row.get(col)
                .unwrap_or(&serde_json::Value::Null);
            format!("`{}` = {}", col, format_value_for_sql(value))
        })
        .collect::<Vec<_>>()
        .join(", ");
    
    Ok(format!(
        "UPDATE `{}` SET {} WHERE `{}` = {};",
        table_name,
        set_clauses,
        key_column,
        format_value_for_sql(key_value)
    ))
}

// Format JSON value to SQL syntax
fn format_value_for_sql(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::Null => "NULL".to_string(),
        serde_json::Value::Bool(b) => if *b { "1" } else { "0" }.to_string(),
        serde_json::Value::Number(n) => n.to_string(),
        serde_json::Value::String(s) => {
            // For strings with special chars, use CAST from hex-encoded BLOB
            // This is the most reliable method for preserving all characters
            if s.contains('\n') || s.contains('\r') || s.contains('\t') {
                // Convert string to hex
                let hex: String = s.as_bytes()
                    .iter()
                    .map(|b| format!("{:02X}", b))
                    .collect();
                format!("CAST(x'{}' AS TEXT)", hex)
            } else {
                // Simple string - just escape single quotes
                format!("'{}'", s.replace("'", "''"))
            }
        },
        _ => {
            let s = value.to_string();
            let escaped = s.replace("'", "''");
            format!("'{}'", escaped)
        }
    }
}

// Get column names from table
// Get a specific column's info from a table
fn get_column_info(conn: &Connection, table_name: &str, column_name: &str) -> Result<ColumnInfo, String> {
    let mut stmt = conn.prepare(&format!("PRAGMA table_info(`{}`)", table_name))
        .map_err(|e| format!("Failed to get table info: {}", e))?;

    let columns: Vec<ColumnInfo> = stmt.query_map([], |row| {
        Ok(ColumnInfo {
            name: row.get(1)?,
            data_type: row.get(2)?,
            is_nullable: row.get::<_, i32>(3)? == 0,
            default_value: row.get(4).ok(),
            is_primary_key: row.get::<_, i32>(5)? == 1,
        })
    })
    .map_err(|e| format!("Failed to query columns: {}", e))?
    .collect::<Result<Vec<_>, _>>()
    .map_err(|e| format!("Failed to collect columns: {}", e))?;

    columns.into_iter()
        .find(|c| c.name == column_name)
        .ok_or_else(|| format!("Column {} not found in table {}", column_name, table_name))
}

fn get_column_names(conn: &Connection, table_name: &str) -> Result<Vec<String>, String> {
    let mut stmt = conn.prepare(&format!("PRAGMA table_info(`{}`)", table_name))
        .map_err(|e| format!("Failed to get table info: {}", e))?;

    let columns: Vec<String> = stmt.query_map([], |row| {
        row.get::<_, String>(1) // Column name is at index 1
    })
    .map_err(|e| format!("Failed to query columns: {}", e))?
    .collect::<Result<Vec<_>, _>>()
    .map_err(|e| format!("Failed to collect columns: {}", e))?;

    Ok(columns)
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
    if parts.len() != 4 || !parts.iter().all(|p| p.len() == 4 && p.chars().all(|c| c.is_alphanumeric())) {
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
        days_remaining: license_info.expiry_date.map(|exp| (exp - Utc::now()).num_days()),
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
        fs::remove_file(&license_file)
            .map_err(|e| format!("Failed to remove license: {}", e))?;
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


async fn verify_license_with_server(email: &str, license_key: &str) -> Result<crate::license::LicenseInfo, String> {
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
        .map_err(|e| format!("License verification failed: {}. Check internet connection.", e))?;

    if !response.status().is_success() {
        return Err("Invalid license key or email".to_string());
    }

    let data: serde_json::Value = response.json().await
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
        Some(DateTime::parse_from_rfc3339(
            data["expiry_date"].as_str().ok_or("Missing expiry date")?
        )
        .map_err(|e| format!("Invalid expiry date: {}", e))?
        .with_timezone(&Utc))
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
    
    fs::write(&license_file, json)
        .map_err(|e| format!("Failed to save license: {}", e))?;
    
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
        let appdata = std::env::var("APPDATA")
            .map_err(|_| "APPDATA environment variable not found")?;
        Ok(PathBuf::from(appdata).join(app_name))
    }
    
    #[cfg(target_os = "macos")]
    {
        let home = std::env::var("HOME")
            .map_err(|_| "HOME environment variable not found")?;
        Ok(PathBuf::from(home)
            .join("Library")
            .join("Application Support")
            .join(app_name))
    }
    
    #[cfg(target_os = "linux")]
    {
        let home = std::env::var("HOME")
            .map_err(|_| "HOME environment variable not found")?;
        Ok(PathBuf::from(home).join(".config").join(app_name))
    }
}

fn get_machine_id_inline() -> String {
    use sha2::{Sha256, Digest};
    
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
) -> Result<MigrationResult, String> 
{
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
            .prepare("SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%'")
            .map_err(|e| format!("Failed to query tables: {}", e))?;

        let rows = stmt.query_map([], |row| row.get(0))
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

        let rows = idx_stmt.query_map([], |row| row.get(0))
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

        let rows = trigger_stmt.query_map([], |row| row.get(0))
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

        let rows = view_stmt.query_map([], |row| row.get(0))
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
                result.push(row.map_err(|e| format!("Failed to read rows from {}: {}", table_name, e))?);
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
    let mut file = std::fs::File::open(&db_path)
        .map_err(|e| format!("Failed to open file: {}", e))?;

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
    let mut conn = Connection::open(&db_path)
        .map_err(|e| format!("Failed to open database: {}", e))?;

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
    let mut verify_conn = Connection::open(&db_path)
        .map_err(|e| format!("Failed to reopen database: {}", e))?;

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

