use crate::git::Git;
use eframe::{
    Frame,
    egui::{Button, CentralPanel, Context, ScrollArea, SidePanel, TopBottomPanel},
};
use std::{
    sync::Arc,
    thread::{self},
};

#[derive(Default)]
pub struct App {
    git: Arc<Git>,
}

impl eframe::App for App {
    fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
        let is_executing = Arc::strong_count(&self.git) > 1;

        TopBottomPanel::top("menu").show(ctx, |ui| {
            if ui.add_enabled(!is_executing, Button::new("pull")).clicked() {
                let git = Arc::clone(&self.git);
                let ctx = ctx.clone();

                thread::spawn(move || {
                    match git.pull() {
                        Ok(output) => print!("{}", String::from_utf8_lossy(&output.stdout)),
                        Err(error) => eprintln!("{}", error),
                    }

                    ctx.request_repaint();
                });
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
    }
}
