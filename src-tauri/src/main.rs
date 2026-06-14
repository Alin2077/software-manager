#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod registry;
mod files;
mod shortcuts;
mod uninstall;
mod app;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([960.0, 680.0])
            .with_min_inner_size([720.0, 480.0])
            .with_title("软件管理器"),
        ..Default::default()
    };

    eframe::run_native(
        "软件管理器",
        options,
        Box::new(|_cc| Ok(Box::new(app::SoftwareManagerApp::default()))),
    )
}
