use std::sync::atomic::{AtomicU32, Ordering};
use tauri::{Listener, Emitter};
use tauri_plugin_macos_input_monitor::{MacOSInputMonitorExt, Modifiers, Hotkey};

// Counter for F5 presses
static F5_PRESS_COUNT: AtomicU32 = AtomicU32::new(0);

#[tauri::command]
fn get_f5_count() -> u32 {
    F5_PRESS_COUNT.load(Ordering::Relaxed)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_macos_input_monitor::init())
        .setup(|app| {
            println!("\n🎯 Example: Overriding F5 dictation shortcut");
            println!("============================================\n");
            
            // Register F5 hotkey using the plugin
            let hotkey = Hotkey {
                keycodes: vec![96, 176], // F5 in both standard and media modes
                modifiers: Modifiers::empty(),
                consume: true, // Block system from seeing F5
                event_name: "f5-pressed".to_string(),
            };
            
            match app.macos_input_monitor().manager.lock() {
                Ok(manager) => {
                    match manager.register(hotkey) {
                        Ok(id) => {
                            println!("✅ F5 hotkey registered! ID: {}", id.0);
                            println!("   Press F5 anywhere - dictation will NOT appear!\n");
                        }
                        Err(e) => {
                            eprintln!("❌ Failed to register hotkey: {}", e);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("❌ Failed to lock manager: {}", e);
                }
            }
            
            // Listen for F5 events from the plugin
            let app_handle = app.handle().clone();
            app.listen("f5-pressed", move |_event| {
                let count = F5_PRESS_COUNT.fetch_add(1, Ordering::Relaxed) + 1;
                println!("🎯 F5 pressed! Count: {}", count);
                
                // Emit to frontend
                let _ = app_handle.emit("f5-count", count);
            });
            
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![get_f5_count])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
