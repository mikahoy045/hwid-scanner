use std::error::Error;
use windows::Win32::System::Com::{CoCreateInstance, CoInitializeEx, CLSCTX_INPROC_SERVER, COINIT_MULTITHREADED};
use windows::Win32::System::Wmi::{IWbemLocator, WbemLocator, WBEM_FLAG_FORWARD_ONLY, WBEM_FLAG_RETURN_IMMEDIATELY, IWbemClassObject};
use windows::core::{BSTR, PCWSTR};

pub fn get_motherboard_info() -> Result<String, Box<dyn Error>> {
    // Initialize COM
    unsafe {
        CoInitializeEx(None, COINIT_MULTITHREADED)?;
    }

    // Create WbemLocator instance
    let locator: IWbemLocator = unsafe {
        CoCreateInstance(&WbemLocator, None, CLSCTX_INPROC_SERVER)?
    };

    // Connect to WMI
    let namespace = BSTR::from("ROOT\\CIMV2");
    let server = BSTR::from("");
    let username = BSTR::from("");
    let password = BSTR::from("");
    let locale = BSTR::from("");
    let authority = BSTR::from("");
    let services = unsafe {
        locator.ConnectServer(
            &namespace,
            &server,
            &username,
            &password,
            0,
            &locale,
            None,
        )?
    };

    // Execute query
    let language = BSTR::from("WQL");
    let query = BSTR::from("SELECT SerialNumber FROM Win32_BaseBoard");
    let flags = WBEM_FLAG_FORWARD_ONLY | WBEM_FLAG_RETURN_IMMEDIATELY;
    let enumerator = unsafe {
        services.ExecQuery(
            &language,
            &query,
            flags,
            None,
        )?
    };

    // Fetch results
    let mut mb_serial = String::new();

    loop {
        let mut item: Option<IWbemClassObject> = None;
        let mut returned = 0;
        
        let result = unsafe { 
            enumerator.Next(
                5000, // 5 second timeout instead of INFINITE
                std::mem::transmute(&mut item), 
                &mut returned,
            )
        };

        if !result.is_ok() || returned == 0 {
            break;
        }

        if let Some(object) = item {
            let variant = unsafe { object.Get(PCWSTR::from_raw(windows::core::w!("SerialNumber").as_ptr()))? };
            if let Some(serial) = variant.to_string() {
                mb_serial = serial;
                break;
            }
        }
    }

    // Fallback to other identifiers if serial number is empty
    if mb_serial.trim().is_empty() {
        // Try to get the motherboard manufacturer + product instead
        let query = BSTR::from("SELECT Manufacturer, Product FROM Win32_BaseBoard");
        let enumerator = unsafe {
            services.ExecQuery(
                &language,
                &query,
                flags,
                None,
            )?
        };

        loop {
            let mut item: Option<IWbemClassObject> = None;
            let mut returned = 0;
            
            let result = unsafe { 
                enumerator.Next(
                    5000, // 5 second timeout instead of INFINITE
                    std::mem::transmute(&mut item), 
                    &mut returned,
                )
            };

            if !result.is_ok() || returned == 0 {
                break;
            }

            if let Some(object) = item {
                let mfr_variant = unsafe { object.Get(PCWSTR::from_raw(windows::core::w!("Manufacturer").as_ptr()))? };
                let prod_variant = unsafe { object.Get(PCWSTR::from_raw(windows::core::w!("Product").as_ptr()))? };
                
                let manufacturer = mfr_variant.to_string().unwrap_or_default();
                let product = prod_variant.to_string().unwrap_or_default();
                
                mb_serial = format!("{}_{}", manufacturer, product);
                break;
            }
        }
    }

    if mb_serial.trim().is_empty() {
        return Err("Could not retrieve motherboard information".into());
    }

    Ok(mb_serial)
} 