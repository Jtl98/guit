use crate::git::Git;
use eframe::{
    Frame,
    egui::{Button, CentralPanel, Context, ScrollArea, SidePanel, TopBottomPanel},
};
use std::{
    sync::{Arc, RwLock},
    thread::{self},
};

#[derive(Default)]
pub struct App {
    git: Arc<Git>,
    files: Arc<RwLock<Vec<String>>>,
    selected_file: Option<String>,
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

                if ui
                    .add_enabled(!is_executing, Button::new("refresh"))
                    .clicked()
                {
                    let git = Arc::clone(&self.git);
                    let files = Arc::clone(&self.files);
                    let ctx = ctx.clone();

                    thread::spawn(move || {
                        match git.diff_name_only() {
                            Ok(output) => {
                                let new_files = output
                                    .stdout
                                    .split(|byte| *byte == b'\n')
                                    .map(|bytes| String::from_utf8_lossy(bytes).to_string())
                                    .collect();

                                let mut files = files.write().unwrap();
                                *files = new_files;
                            }
                            Err(error) => eprintln!("{}", error),
                        }

                        ctx.request_repaint();
                    });
                }

                for file in self.files.read().unwrap().iter() {
                    ui.selectable_value(&mut self.selected_file, Some(file.to_owned()), file);
                }
            });
        });

        CentralPanel::default().show(ctx, |ui| {
            ui.heading("diff");
        });
    }
}
