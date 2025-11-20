// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod models;
mod database;
mod commands;
mod license;

use database::DatabaseManager;
use license::LicenseManager;
use std::sync::Mutex;
use tauri_plugin_dialog;

fn main() {
    // Initialize database manager
    let db_manager = Mutex::new(DatabaseManager::new());

    // Initialize license manager
       let license_manager = match LicenseManager::new() {
        Ok(manager) => Mutex::new(manager),
        Err(e) => {
            eprintln!("Failed to initialize license manager: {}", e);
            std::process::exit(1);
        }
    };
    
    tauri::Builder::default()
        // .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_opener::init())
        .manage(db_manager)
        .manage(license_manager)
        .invoke_handler(tauri::generate_handler![
            commands::test_connection,
            commands::connect_database,
            commands::get_database_tables,
            commands::get_table_data,
            commands::get_table_info,
            commands::compare_database_schemas,
            commands::compare_table_data_fast,
            commands::generate_schema_patch,
            commands::apply_schema_patch,
            commands::generate_data_patch,
            commands::apply_data_patch,
            commands::get_license_status,
            commands::activate_license,
            commands::deactivate_license,
            commands::check_trial_status,
            commands::migrate_to_sqlcipher,
            commands::rekey_sqlcipher_database,
            commands::check_database_type,
        ])
        .run(tauri::generate_context!())
                .expect("error while running tauri application");
         
        }

        