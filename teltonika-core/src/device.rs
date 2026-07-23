use std::collections::HashMap;
use serde::Serialize;

#[derive(Serialize)]
pub struct Credentials {
    pub username: String,
    pub password: String,
}

impl Credentials {
    pub fn from_json(json: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let credentials: Credentials = serde_json::from_str(json)?;
        Ok(credentials)
    }
    pub fn from_yaml(yaml: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let credentials: Credentials = serde_yaml::from_str(yaml)?;
        Ok(credentials)
    }
    pub fn from_env() -> Result<Self, Box<dyn std::error::Error>> {
        let username = std::env::var("DEVICE_USERNAME")?;
        let password = std::env::var("DEVICE_PASSWORD")?;
        Ok(Credentials { username, password })
    }
}

struct DeviceType {
    pub model: String,
    pub firmware_version: String,
}

impl DeviceType {
    pub fn new(model: String, firmware_version: String) -> Self {
        DeviceType {
            model,
            firmware_version,
        }
    }
    fn available_device_types() -> HashMap<String, String> {
        todo!("Implement available device types for further validation. Get device types from database or configuration file.");
        let device_types = HashMap::new();
        device_types.insert("RUTX50".to_string(), "RUTX_R_00.07.23.7".to_string());
        device_types
    }
    fn validate_device_type(model: &str, firmware_version: &str) -> bool {
        let device_types = DeviceType::available_device_types();
        if let Some(expected_firmware) = device_types.get(model) {
            return expected_firmware == firmware_version;
        }
        false
    }
    
}

pub struct Device {
    pub device_type: DeviceType,
    pub ip: String,
    pub credentials: Credentials,
}

impl Device {
    pub fn new(model: String, firmware_version: String, ip: String, credentials: Credentials) -> Self {
        if !DeviceType::validate_device_type(&model, &firmware_version) {
            panic!("Invalid device type: model '{}' with firmware version '{}'.
                    Available device types are: {:?}", 
                    model, firmware_version, DeviceType::available_device_types());
        }
        if !Device::validate_ip(&ip) {
            panic!("Invalid IP address: '{}'. Please provide a valid IPv4 address.", ip);
        }
        Device {
            device_type: DeviceType {
                model,
                firmware_version,
            },
            ip,
            credentials
        }
    }
    fn validate_ip(ip: &str) -> bool {
        // Simple IP address validation (IPv4)
        let segments: Vec<&str> = ip.split('.').collect();
        if segments.len() != 4 {
            return false;
        }
        for segment in segments {
            if let Ok(num) = segment.parse::<u8>() {
                if num > 255 {
                    return false;
                }
            } else {
                return false;
            }
        }
        true
    }
}


