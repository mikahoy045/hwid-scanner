use eframe::egui;
use sha2::{Digest, Sha256};
use std::error::Error;

#[cfg(target_os = "windows")]
mod windows_hwid;

#[cfg(not(target_os = "windows"))]
mod unix_hwid;

fn main() -> Result<(), Box<dyn Error>> {
    #[cfg(target_os = "linux")]
    {
        let args: Vec<String> = std::env::args().collect();
        if args.iter().any(|arg| arg == "--headless") {
            println!("{}", get_motherboard_hwid()?);
            return Ok(());
        }
    }

    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(400.0, 300.0)),
        ..Default::default()
    };
    
    let _ = eframe::run_native(
        "Speramus HWID",
        options,
        Box::new(|_cc| Box::new(HWIDApp::new())),
    );
    
    Ok(())
}

struct HWIDApp {
    hwid: String,
    error_message: Option<String>,
}

impl HWIDApp {
    fn new() -> Self {
        let (hwid, error_message) = match get_motherboard_hwid() {
            Ok(id) => (id, None),
            Err(e) => (String::new(), Some(e.to_string())),
        };
        
        Self {
            hwid,
            error_message,
        }
    }
}

impl eframe::App for HWIDApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Hardware ID Information");
            
            if let Some(error) = &self.error_message {
                ui.label(format!("Error: {}", error));
            } else {
                ui.label("Your unique motherboard hardware ID:");
                
                ui.add(egui::TextEdit::multiline(&mut self.hwid.clone())
                    .desired_width(f32::INFINITY)
                    .font(egui::TextStyle::Monospace));
                
                if ui.button("Copy to Clipboard").clicked() {
                    ui.output_mut(|o| o.copied_text = self.hwid.clone());
                }
                
                ui.separator();
                ui.label("This ID is unique to your motherboard and will change if your motherboard is replaced.");
            }
        });
    }
}

fn get_motherboard_hwid() -> Result<String, Box<dyn Error>> {
    #[cfg(target_os = "windows")]
    let raw_hwid = windows_hwid::get_motherboard_info()?;
    
    #[cfg(not(target_os = "windows"))]
    let raw_hwid = unix_hwid::get_motherboard_info()?;
    
    // Create a hash of the hardware information to make it consistent length
    let mut hasher = Sha256::new();
    hasher.update(raw_hwid);
    let result = hasher.finalize();
    
    // Convert to hex string
    let hwid = hex::encode(result);
    
    Ok(hwid)
}
