#![forbid(unsafe_code)]
#![cfg_attr(not(debug_assertions), deny(warnings))] // Forbid warnings in release builds
#![warn(clippy::all, rust_2018_idioms)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] //Hide console window in release builds on Windows, this blocks stdout.

// When compiling natively:
#[cfg(not(target_arch = "wasm32"))]
fn main() {
    // let app = eframe_template::TemplateApp::default();

    use diadro::TemplateApp;
    let options = eframe::NativeOptions {
        // Let's show off that we support transparent windows
        // transparent: true,
        // drag_and_drop_support: true,
        ..Default::default()
    };

    eframe::run_native(
        "egui demo app",
        options,
        Box::new(|_| Box::new(TemplateApp::default())),
    );
}
