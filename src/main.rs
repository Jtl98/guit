mod app;
mod common;
mod git;
mod repo;

use crate::app::App;
use eframe::NativeOptions;

fn main() -> eframe::Result {
    let options = NativeOptions {
        ..Default::default()
    };

    eframe::run_native("guit", options, Box::new(|_cc| Ok(Box::<App>::default())))
}
