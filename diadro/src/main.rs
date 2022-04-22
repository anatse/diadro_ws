#![forbid(unsafe_code)]
#![cfg_attr(not(debug_assertions), deny(warnings))] // Forbid warnings in release builds
#![warn(clippy::all)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] //Hide console window in release builds on Windows, this blocks stdout.

// When compiling natively:
#[cfg(not(target_arch = "wasm32"))]
fn main() {
    use diadro::TemplateApp;
    let options = eframe::NativeOptions {
        ..Default::default()
    };

    eframe::run_native(
        "egui demo app",
        options,
        Box::new(|_| Box::new(TemplateApp::default())),
    );
}
