mod app;

use crate::app::App;

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        ..Default::default()
    };

    eframe::run_native("guit", options, Box::new(|_cc| Ok(Box::<App>::default())))
}
