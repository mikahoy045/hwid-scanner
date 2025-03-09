use std::error::Error;
use std::fs;
use std::process::Command;

pub fn get_motherboard_info() -> Result<String, Box<dyn Error>> {
    let mut unique_id = String::new();
    
    // Try to get motherboard information from DMI on Linux systems
    if let Ok(output) = Command::new("dmidecode")
        .args(["-t", "baseboard"])
        .output()
    {
        if output.status.success() {
            let output_str = String::from_utf8_lossy(&output.stdout);
            
            // Extract serial number
            if let Some(serial_line) = output_str.lines().find(|l| l.contains("Serial Number:")) {
                if let Some(serial) = serial_line.split(':').nth(1) {
                    unique_id.push_str(serial.trim());
                }
            }
            
            // Extract manufacturer
            if let Some(mfg_line) = output_str.lines().find(|l| l.contains("Manufacturer:")) {
                if let Some(mfg) = mfg_line.split(':').nth(1) {
                    unique_id.push_str(mfg.trim());
                }
            }
            
            // Extract product name
            if let Some(product_line) = output_str.lines().find(|l| l.contains("Product Name:")) {
                if let Some(product) = product_line.split(':').nth(1) {
                    unique_id.push_str(product.trim());
                }
            }
        }
    }
    
    // If we couldn't get info from dmidecode, try reading from sysfs on Linux
    if unique_id.is_empty() && std::path::Path::new("/sys/class/dmi/id").exists() {
        // Read board_serial
        if let Ok(serial) = fs::read_to_string("/sys/class/dmi/id/board_serial") {
            unique_id.push_str(serial.trim());
        }
        
        // Read board_vendor
        if let Ok(vendor) = fs::read_to_string("/sys/class/dmi/id/board_vendor") {
            unique_id.push_str(vendor.trim());
        }
        
        // Read board_name
        if let Ok(name) = fs::read_to_string("/sys/class/dmi/id/board_name") {
            unique_id.push_str(name.trim());
        }
    }
    
    // For macOS, try to use system_profiler
    if unique_id.is_empty() {
        if let Ok(output) = Command::new("system_profiler")
            .arg("SPHardwareDataType")
            .output()
        {
            if output.status.success() {
                let output_str = String::from_utf8_lossy(&output.stdout);
                
                // Extract hardware UUID
                if let Some(uuid_line) = output_str.lines().find(|l| l.contains("Hardware UUID:")) {
                    if let Some(uuid) = uuid_line.split(':').nth(1) {
                        unique_id.push_str(uuid.trim());
                    }
                }
                
                // Extract serial number
                if let Some(serial_line) = output_str.lines().find(|l| l.contains("Serial Number")) {
                    if let Some(serial) = serial_line.split(':').nth(1) {
                        unique_id.push_str(serial.trim());
                    }
                }
            }
        }
    }
    
    // If we still don't have an ID, try some fallback methods
    if unique_id.is_empty() {
        // Try to get machine-id as a last resort
        if let Ok(machine_id) = fs::read_to_string("/etc/machine-id") {
            unique_id.push_str(machine_id.trim());
        } else if let Ok(machine_id) = fs::read_to_string("/var/lib/dbus/machine-id") {
            unique_id.push_str(machine_id.trim());
        }
    }
    
    if unique_id.is_empty() {
        return Err("Failed to retrieve hardware identification".into());
    }
    
    Ok(unique_id)
} 