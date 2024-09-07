#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use crate::ui::AutoShooter;
use eframe::egui;

mod ui;

#[tokio::main]
async fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([240.0, 320.0]),
        ..Default::default()
    };

    eframe::run_native(
        "自動連射ツール",
        options,
        Box::new(|cc| Ok(Box::new(AutoShooter::new(cc)))),
    )
}
