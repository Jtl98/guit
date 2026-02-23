use crate::git::Git;
use eframe::{
    Frame,
    egui::{Button, CentralPanel, Context, ScrollArea, SidePanel, TopBottomPanel},
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

        SidePanel::left("files").show(ctx, |ui| {
            ScrollArea::both().show(ui, |ui| {
                ui.take_available_space();
                ui.heading("files");
            });
        });

        CentralPanel::default().show(ctx, |ui| {
            ui.heading("diff");
        });

        self.git.update();
    }
}
