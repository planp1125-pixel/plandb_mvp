// src-tauri/src/license.rs
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use chrono::{DateTime, Utc, Duration};
use sha2::{Sha256, Digest};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LicenseInfo {
    pub email: String,
    pub license_key: String,
    pub license_type: LicenseType,
    pub activation_date: DateTime<Utc>,
    pub expiry_date: Option<DateTime<Utc>>,
    pub last_validation: DateTime<Utc>,
    pub machine_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum LicenseType {
    Trial,
    Monthly,
    Yearly,
    Lifetime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LicenseStatus {
    pub is_valid: bool,
    pub license_type: LicenseType,
    pub days_remaining: Option<i64>,
    pub message: String,
}

pub struct LicenseManager {
    license_file: PathBuf,
}

impl LicenseManager {
    pub fn new() -> Result<Self, String> {
        let app_data_dir = get_app_data_dir()?;
        fs::create_dir_all(&app_data_dir)
            .map_err(|e| format!("Failed to create app data directory: {}", e))?;
        
        Ok(LicenseManager {
            license_file: app_data_dir.join("license.json"),
        })
    }

    pub async fn activate_license(&self, email: &str, license_key: &str) -> Result<LicenseStatus, String> {
        // Validate format
        if !self.validate_license_format(license_key) {
            return Ok(LicenseStatus {
                is_valid: false,
                license_type: LicenseType::Trial,
                days_remaining: None,
                message: "Invalid license key format".to_string(),
            });
        }

        // Verify with server
        let license_info = self.verify_with_server(email, license_key).await?;
        
        // Save license locally
        self.save_license(&license_info)?;
        
        Ok(self.get_license_status()?)
    }

    pub fn get_license_status(&self) -> Result<LicenseStatus, String> {
        match self.load_license() {
            Ok(license) => {
                let now = Utc::now();

                // Check expiry
                if let Some(expiry) = license.expiry_date {
                    if now > expiry {
                        return Ok(LicenseStatus {
                            is_valid: false,
                            license_type: license.license_type,
                            days_remaining: Some(0),
                            message: "License expired".to_string(),
                        });
                    }

                    let days_remaining = (expiry - now).num_days();
                    
                    Ok(LicenseStatus {
                        is_valid: true,
                        license_type: license.license_type.clone(),
                        days_remaining: Some(days_remaining),
                        message: format!("License active - {} license", 
                            match license.license_type {
                                LicenseType::Trial => "Trial",
                                LicenseType::Monthly => "Monthly",
                                LicenseType::Yearly => "Yearly",
                                LicenseType::Lifetime => "Lifetime",
                            }
                        ),
                    })
                } else {
                    // Lifetime license
                    Ok(LicenseStatus {
                        is_valid: true,
                        license_type: LicenseType::Lifetime,
                        days_remaining: None,
                        message: "Lifetime license active".to_string(),
                    })
                }
            }
            Err(_) => {
                // No license - create trial
                self.create_trial_license()?;
                self.get_license_status()
            }
        }
    }

    fn create_trial_license(&self) -> Result<(), String> {
        let now = Utc::now();
        let trial_license = LicenseInfo {
            email: "trial@user.local".to_string(),
            license_key: "TRIAL-LICENSE".to_string(),
            license_type: LicenseType::Trial,
            activation_date: now,
            expiry_date: Some(now + Duration::days(14)),
            last_validation: now,
            machine_id: get_machine_id(),
        };

        self.save_license(&trial_license)
    }

    fn validate_license_format(&self, key: &str) -> bool {
        if key.len() != 19 {
            return false;
        }

        let parts: Vec<&str> = key.split('-').collect();
        if parts.len() != 4 {
            return false;
        }

        parts.iter().all(|part| {
            part.len() == 4 && part.chars().all(|c| c.is_alphanumeric())
        })
    }

    async fn verify_with_server(&self, email: &str, license_key: &str) -> Result<LicenseInfo, String> {
        let client = reqwest::Client::new();
        let machine_id = get_machine_id();
        
        // TODO: Replace with your actual license server URL
       // let server_url = "https://your-license-server.vercel.app/api/verify";
      //  let server_url = "http://localhost:3000/api/verify";
      let server_url = "https://plandbdiff-7qwur9pu9-manikandans-projects-be37ef3a.vercel.app/api/verify";
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
            Some("monthly") => LicenseType::Monthly,
            Some("yearly") => LicenseType::Yearly,
            Some("lifetime") => LicenseType::Lifetime,
            _ => return Err("Invalid license type from server".to_string()),
        };

        let expiry_date = if license_type == LicenseType::Lifetime {
            None
        } else {
            Some(DateTime::parse_from_rfc3339(
                data["expiry_date"].as_str().ok_or("Missing expiry date")?
            )
            .map_err(|e| format!("Invalid expiry date: {}", e))?
            .with_timezone(&Utc))
        };

        Ok(LicenseInfo {
            email: email.to_string(),
            license_key: license_key.to_string(),
            license_type,
            activation_date: Utc::now(),
            expiry_date,
            last_validation: Utc::now(),
            machine_id,
        })
    }

    fn save_license(&self, license: &LicenseInfo) -> Result<(), String> {
        let json = serde_json::to_string_pretty(license)
            .map_err(|e| format!("Failed to serialize license: {}", e))?;
        
        fs::write(&self.license_file, json)
            .map_err(|e| format!("Failed to save license: {}", e))?;
        
        Ok(())
    }

    fn load_license(&self) -> Result<LicenseInfo, String> {
        let json = fs::read_to_string(&self.license_file)
            .map_err(|e| format!("Failed to read license file: {}", e))?;
        
        serde_json::from_str(&json)
            .map_err(|e| format!("Failed to parse license: {}", e))
    }

    pub fn deactivate_license(&self) -> Result<(), String> {
        if self.license_file.exists() {
            fs::remove_file(&self.license_file)
                .map_err(|e| format!("Failed to remove license: {}", e))?;
        }
        Ok(())
    }
}

fn get_machine_id() -> String {
    let mut hasher = Sha256::new();
    
    // Add hostname
    if let Ok(hostname) = hostname::get() {
        hasher.update(hostname.to_string_lossy().as_bytes());
    }
    
    // Add MAC address
    if let Ok(Some(ma)) = mac_address::get_mac_address() {
        hasher.update(ma.to_string().as_bytes());
    }
    
    // Add OS info
    hasher.update(std::env::consts::OS.as_bytes());
    
    let result = hasher.finalize();
    format!("{:x}", result)
}

fn get_app_data_dir() -> Result<PathBuf, String> {
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