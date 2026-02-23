use crate::git::Git;
use eframe::{
    Frame,
    egui::{Button, CentralPanel, Context, TopBottomPanel},
};

#[derive(Default)]
pub struct App {
    git: Git,
}

impl eframe::App for App {
    fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
        let is_executing = self.git.is_executing();

        TopBottomPanel::top("menu").show(ctx, |ui| {
            if ui.add_enabled(!is_executing, Button::new("pull")).clicked() {
                self.git.pull();
            }
        });

        CentralPanel::default().show(ctx, |ui| {
            ui.heading("guit");
        });

        self.git.update();
    }
}
