const COMMANDS: &[&str] = &[
    "register",
    "unregister",
    "is_registered",
    "get_keycode_table",
    "discover_keycode",
    "open_input_monitoring_settings",
    "check_permission",
];

fn main() {
  // Link AudioToolbox framework for macOS system sounds (if needed by plugin users)
  #[cfg(target_os = "macos")]
  println!("cargo:rustc-link-lib=framework=AudioToolbox");
  
  tauri_plugin::Builder::new(COMMANDS)
    .build();
}
