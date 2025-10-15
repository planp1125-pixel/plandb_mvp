// // backend/src/license_manager.rs
// use serde::{Deserialize, Serialize};
// use std::fs;
// use std::path::PathBuf;
// use anyhow::Result;
// use std::time::{SystemTime, Duration};

// #[derive(Serialize, Deserialize, Clone)]
// pub struct LicenseData {
//     pub license_key: String,
//     pub license_type: String, // "trial", "monthly", "yearly", "lifetime"
//     pub activated_at: u64,
//     pub valid_until: u64,
//     pub status: String, // "active", "expired", "cancelled"
//     pub lemon_squeezy_order_id: Option<String>,
// }

// pub struct LicenseManager {
//     app_data_dir: PathBuf,
// }

// impl LicenseManager {
//     pub fn new(app_data_dir: PathBuf) -> Self {
//         Self { app_data_dir }
//     }
    
//     pub fn initialize_trial(&self) -> Result<LicenseData> {
//         let current_time = SystemTime::now()
//             .duration_since(SystemTime::UNIX_EPOCH)?
//             .as_secs();
            
//         let trial_license = LicenseData {
//             license_key: "TRIAL".to_string(),
//             license_type: "trial".to_string(),
//             activated_at: current_time,
//             valid_until: current_time + (14 * 24 * 60 * 60), // 14 days
//             status: "active".to_string(),
//             lemon_squeezy_order_id: None,
//         };
        
//         self.save_license(&trial_license)?;
//         Ok(trial_license)
//     }
    
//     pub fn activate_license(&self, license_key: &str, order_id: &str, license_type: &str) -> Result<LicenseData> {
//         let current_time = SystemTime::now()
//             .duration_since(SystemTime::UNIX_EPOCH)?
//             .as_secs();
            
//         let valid_until = match license_type {
//             "monthly" => current_time + (30 * 24 * 60 * 60),
//             "yearly" => current_time + (365 * 24 * 60 * 60),
//             "lifetime" => current_time + (10 * 365 * 24 * 60 * 60), // 10 years
//             _ => current_time + (30 * 24 * 60 * 60), // default to monthly
//         };
        
//         let license = LicenseData {
//             license_key: license_key.to_string(),
//             license_type: license_type.to_string(),
//             activated_at: current_time,
//             valid_until,
//             status: "active".to_string(),
//             lemon_squeezy_order_id: Some(order_id.to_string()),
//         };
        
//         self.save_license(&license)?;
//         Ok(license)
//     }
    
//     pub fn check_license_status(&self) -> Result<LicenseStatus> {
//         let license = self.load_license()?;
//         let current_time = SystemTime::now()
//             .duration_since(SystemTime::UNIX_EPOCH)?
//             .as_secs();
            
//         let days_remaining = if license.valid_until > current_time {
//             Some((license.valid_until - current_time) / (24 * 60 * 60))
//         } else {
//             Some(0)
//         };
        
//         let is_valid = license.status == "active" && license.valid_until > current_time;
        
//         Ok(LicenseStatus {
//             is_valid,
//             license_type: match license.license_type.as_str() {
//                 "trial" => "Trial",
//                 "monthly" => "Monthly", 
//                 "yearly" => "Yearly",
//                 "lifetime" => "Lifetime",
//                 _ => "Trial"
//             }.to_string(),
//             days_remaining,
//             message: if is_valid {
//                 format!("{} license active", license.license_type)
//             } else {
//                 "License expired or invalid".to_string()
//             },
//         })
//     }
    
//     fn save_license(&self, license: &LicenseData) -> Result<()> {
//         let license_path = self.app_data_dir.join("license.json");
//         let serialized = serde_json::to_string_pretty(license)?;
//         fs::write(license_path, serialized)?;
//         Ok(())
//     }
    
//     fn load_license(&self) -> Result<LicenseData> {
//         let license_path = self.app_data_dir.join("license.json");
//         if license_path.exists() {
//             let content = fs::read_to_string(license_path)?;
//             let license: LicenseData = serde_json::from_str(&content)?;
//             Ok(license)
//         } else {
//             // Initialize trial if no license exists
//             self.initialize_trial()
//         }
//     }
// }

// #[derive(Serialize)]
// pub struct LicenseStatus {
//     pub is_valid: bool,
//     pub license_type: String,
//     pub days_remaining: Option<u64>,
//     pub message: String,
// }