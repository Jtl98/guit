use crate::git::Git;
use eframe::egui::{self, Button};

#[derive(Default)]
pub struct App {
    git: Git,
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            let is_executing = self.git.is_executing();

            ui.heading("guit");

            if ui.add_enabled(!is_executing, Button::new("pull")).clicked() {
                self.git.pull();
            }

            self.git.update();
        });
    }
}
