use std::error::Error;
use wmi::{COMLibrary, Variant, WMIConnection};

pub fn get_motherboard_info() -> Result<String, Box<dyn Error>> {
    // Initialize COM library
    let com_con = COMLibrary::new()?;
    
    // Connect to WMI
    let wmi_con = WMIConnection::new(com_con)?;
    
    // Query for motherboard (baseboard) information
    let baseboard: Vec<std::collections::HashMap<String, Variant>> = wmi_con.raw_query(
        "SELECT Manufacturer, Product, SerialNumber FROM Win32_BaseBoard"
    )?;
    
    // Query for BIOS information for additional uniqueness
    let bios: Vec<std::collections::HashMap<String, Variant>> = wmi_con.raw_query(
        "SELECT Manufacturer, SerialNumber, Version FROM Win32_BIOS"
    )?;
    
    // Query for processor ID which can be useful for uniqueness
    let processor: Vec<std::collections::HashMap<String, Variant>> = wmi_con.raw_query(
        "SELECT ProcessorId FROM Win32_Processor"
    )?;
    
    // Combine all information to create a unique identifier
    let mut unique_id = String::new();
    
    // Extract motherboard information
    if let Some(board) = baseboard.first() {
        if let Some(Variant::String(manufacturer)) = board.get("Manufacturer") {
            unique_id.push_str(manufacturer);
        }
        
        if let Some(Variant::String(product)) = board.get("Product") {
            unique_id.push_str(product);
        }
        
        if let Some(Variant::String(serial)) = board.get("SerialNumber") {
            unique_id.push_str(serial);
        }
    }
    
    // Extract BIOS information
    if let Some(bios_info) = bios.first() {
        if let Some(Variant::String(serial)) = bios_info.get("SerialNumber") {
            unique_id.push_str(serial);
        }
    }
    
    // Extract processor information as a backup uniqueness factor
    if let Some(proc_info) = processor.first() {
        if let Some(Variant::String(proc_id)) = proc_info.get("ProcessorId") {
            unique_id.push_str(proc_id);
        }
    }
    
    // Make sure the ID isn't empty
    if unique_id.is_empty() {
        return Err("Failed to retrieve any hardware identification".into());
    }
    
    Ok(unique_id)
} 