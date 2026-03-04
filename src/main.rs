mod app;
mod common;
mod config;
mod git;
mod log;
mod repo;

use crate::app::App;
use eframe::{NativeOptions, egui::ViewportBuilder};

fn main() -> eframe::Result {
    log::init();

    let options = NativeOptions {
        viewport: ViewportBuilder::default().with_maximized(true),
        ..Default::default()
    };

    eframe::run_native("guit", options, Box::new(|_cc| Ok(Box::<App>::default())))
}
