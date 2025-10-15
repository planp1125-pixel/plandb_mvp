// use crate::database::DatabaseManager;
// use crate::models::*;
// use std::sync::Mutex;
// use std::time::Duration;
// use tauri::State;
// use tokio::time::timeout;
// use rusqlite::Connection;
// use std::fs;
// use std::path::PathBuf;
// use chrono::{Utc, Duration as ChronoDuration};
// use sha2::{Sha256, Digest};
// //use rusqlite::{Connection, types::Value};
// //use crate::models::SchemaComparison;
// use crate::license::{LicenseManager, LicenseStatus};


use crate::database::DatabaseManager;
use crate::models::*;
use std::sync::Mutex;
use std::time::Duration;
use tauri::State;
use tokio::time::timeout;
use rusqlite::Connection;

use crate::license::{LicenseManager, LicenseStatus};
use std::fs;
use std::path::PathBuf;
use chrono::{DateTime, Utc};  // ← Add DateTime here
use sha2::{Sha256, Digest};
// ADD THIS TYPE ALIAS AFTER YOUR IMPORTS
type LicenseManagerState = Mutex<LicenseManager>;
type DbManager = Mutex<DatabaseManager>;

#[tauri::command]
pub async fn connect_database(
    path: String,
    password: String,
    manager: State<'_, DbManager>,
) -> Result<DatabaseInfo, String> {
    let mut db_manager = manager.lock().unwrap();
    
    match db_manager.connect_database(&path, &password) {
        Ok(db_info) => {
            println!("Successfully connected to database: {}", path);
            Ok(db_info)
        },
        Err(e) => {
            println!("Failed to connect to database {}: {}", path, e);
            Err(format!("Connection failed: {}", e))
        }
    }
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
    manager: State<'_, DbManager>,
) -> Result<TableData, String> {
    // Add timeout for large queries (60 seconds)
    let timeout_duration = Duration::from_secs(60);
    
    let result = timeout(timeout_duration, async {
        let db_manager = manager.lock().unwrap();
        
        // Log the request with more details
        println!("Fetching table '{}' from '{}' with limit: {:?}", 
                table_name, db_path, limit);
        
        db_manager.get_table_data(&db_path, &table_name, limit)
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
            println!("Timeout occurred while fetching table '{}' (limit: {:?})", 
                    table_name, limit);
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
    
    sql.push_str("BEGIN TRANSACTION;\n\n");
    
    // Drop removed tables
    for table in &comparison.removed_tables {
        sql.push_str("-- Drop removed table\n");
        sql.push_str(&format!("DROP TABLE IF EXISTS `{}`;\n\n", table));
    }
    
    // Create added tables
    for table in &comparison.added_tables {
        match get_create_table_sql(&target_conn, table) {
            Ok(create_sql) => {
                sql.push_str("-- Create new table\n");
                sql.push_str(&format!("{};\n\n", create_sql));
            }
            Err(e) => {
                sql.push_str(&format!("-- ERROR: Could not get schema for {}: {}\n\n", table, e));
            }
        }
    }
    
    // Modify existing tables
    for modified in &comparison.modified_tables {
        let needs_recreation = !modified.removed_columns.is_empty() 
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
                    if !modified.removed_columns.is_empty() {
                        sql.push_str(&format!("--   Removed columns: {}\n", 
                            modified.removed_columns.join(", ")));
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
        } else {
            // Only added columns - simple ALTER TABLE
            sql.push_str(&format!("-- Modify table: {} (add columns)\n", modified.table_name));
            for col in &modified.added_columns {
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
            sql.push_str("\n");
        }
    }
    
    sql.push_str("COMMIT;\n");
    
    Ok(sql)
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
    // Get the target schema (what we want)
    let target_schema = get_create_table_sql(target_conn, table_name)
        .map_err(|e| format!("Could not get target schema: {}", e))?;
    
    // Get columns from target (final state)
    let target_columns = get_table_column_names(target_conn, table_name)?;
    
    // Get columns from source (current state)
    let source_columns = get_table_column_names(source_conn, table_name)?;
    
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
    
    // Step 1: Disable foreign keys
    sql.push_str("PRAGMA foreign_keys=off;\n\n");
    
    // Step 2: Start transaction (already in one, but document it)
    sql.push_str("-- Transaction already started\n\n");
    
    // Step 3: Rename old table
    sql.push_str(&format!("ALTER TABLE `{}` RENAME TO `{}_old`;\n\n", table_name, table_name));
    
    // Step 4: Create new table with target schema
    sql.push_str(&format!("{};\n\n", target_schema));
    
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
    
    // Step 7: Re-enable foreign keys
    sql.push_str("PRAGMA foreign_keys=on;\n\n");
    
    // Add comment about what changed
    sql.push_str("-- Changes applied:\n");
    if !diff.removed_columns.is_empty() {
        sql.push_str(&format!("--   Removed: {}\n", diff.removed_columns.join(", ")));
    }
    if !diff.added_columns.is_empty() {
        let added: Vec<String> = diff.added_columns.iter().map(|c| c.name.clone()).collect();
        sql.push_str(&format!("--   Added: {}\n", added.join(", ")));
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
    let manager = db_manager.lock().unwrap();
    
    // Open both databases
    let source_conn = Connection::open(&db1_path)
        .map_err(|e| format!("Failed to open source database: {}", e))?;
    
    let target_conn = Connection::open(&db2_path)
        .map_err(|e| format!("Failed to open target database: {}", e))?;

    // Unlock databases if needed
    let db1_password = manager.get_password(&db1_path).unwrap_or_default();
    let db2_password = manager.get_password(&db2_path).unwrap_or_default();
    
    unlock_database(&source_conn, &db1_password)?;
    unlock_database(&target_conn, &db2_password)?;

    // Generate SQL patch
    let mut sql = String::new();
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
                
                for row in missing {
                    let insert_sql = generate_insert_statement(
                        table_name, 
                        &columns, 
                        row
                    )?;
                    sql.push_str(&insert_sql);
                    sql.push_str("\n");
                }
                sql.push_str("\n");
            }
        }

        // 2. UPDATE different rows
        if let Some(different) = comparison.get("differentRows").and_then(|v| v.as_array()) {
            if !different.is_empty() {
                sql.push_str(&format!("-- UPDATE {} different rows in {}\n", different.len(), table_name));
                
                for diff in different {
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
                }
                sql.push_str("\n");
            }
        }

        // 3. DELETE extra rows (exist in target, not in source) - COMMENTED OUT
        if let Some(extra) = comparison.get("extraInTarget").and_then(|v| v.as_array()) {
            if !extra.is_empty() {
                sql.push_str(&format!("-- DELETE {} extra rows from {} (COMMENTED FOR SAFETY)\n", 
                    extra.len(), table_name));
                sql.push_str("-- Uncomment the following lines if you want to remove these rows:\n\n");
                
                for row in extra {
                    let key_value = row.get(key_column)
                        .ok_or_else(|| format!("Missing key column {} in row", key_column))?;
                    
                    let delete_sql = format!(
                        "-- DELETE FROM `{}` WHERE `{}` = {};\n",
                        table_name,
                        key_column,
                        format_value_for_sql(key_value)
                    );
                    sql.push_str(&delete_sql);
                }
                sql.push_str("\n");
            }
        }
    }
    
    sql.push_str("COMMIT;\n\n");
    sql.push_str("-- End of data synchronization patch\n");
    
    Ok(sql)
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
            // Escape single quotes by doubling them (SQL standard)
            let escaped = s.replace("'", "''");
            format!("'{}'", escaped)
        },
        _ => format!("'{}'", value.to_string().replace("'", "''"))
    }
}

// Get column names from table
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
    use sha2::{Sha256, Digest};
    
    let client = reqwest::Client::new();
    let machine_id = get_machine_id_inline();
    
    let server_url = "http://localhost:3000/api/verify";
    
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