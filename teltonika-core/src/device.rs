use std::collections::HashMap;

#[allow(dead_code)]
struct DeviceType {
    pub model: String,
    pub firmware_version: String,
}

impl DeviceType {
    #[allow(dead_code)]
    pub fn new(model: String, firmware_version: String) -> Self {
        DeviceType {
            model,
            firmware_version,
        }
    }
    #[allow(dead_code)]
    fn available_device_types() -> HashMap<String, String> {
        //TODO: Implement available device types for further validation. Get device types from database or configuration file
        let mut device_types = HashMap::new();
        device_types.insert("RUTX50".to_string(), "RUTX_R_00.07.23.7".to_string());
        device_types
    }
    #[allow(dead_code)]
    fn validate_device_type(model: &str, firmware_version: &str) -> bool {
        let device_types = DeviceType::available_device_types();
        if let Some(expected_firmware) = device_types.get(model) {
            return expected_firmware == firmware_version;
        }
        false
    }
    
}