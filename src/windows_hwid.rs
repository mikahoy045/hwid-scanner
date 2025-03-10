use std::error::Error;
use wmi::{COMLibrary, WMIConnection};
use serde::Deserialize;

#[derive(Deserialize, Debug)]
struct BaseBoard {
    SerialNumber: Option<String>,
    Manufacturer: Option<String>,
    Product: Option<String>,
}

pub fn get_motherboard_info() -> Result<String, Box<dyn Error>> {
    // Initialize COM library
    let com_lib = COMLibrary::new()?;
    
    // Connect to WMI
    let wmi_con = WMIConnection::new(com_lib)?;
    
    // First try to get the motherboard serial number
    let results: Vec<BaseBoard> = wmi_con.query()?;
    
    for board in &results {
        if let Some(serial) = &board.SerialNumber {
            if !serial.trim().is_empty() && !serial.trim().eq("To be filled by O.E.M.") {
                return Ok(serial.clone());
            }
        }
    }
    
    // Fall back to manufacturer + product
    for board in &results {
        let manufacturer = board.Manufacturer.as_deref().unwrap_or("Unknown").trim();
        let product = board.Product.as_deref().unwrap_or("Unknown").trim();
        
        if manufacturer != "Unknown" || product != "Unknown" {
            return Ok(format!("{}_{}", manufacturer, product));
        }
    }
    
    Err("Could not retrieve valid motherboard information".into())
} 