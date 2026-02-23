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
    paths: Arc<RwLock<Vec<String>>>,
    selected_path: Option<String>,
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

        SidePanel::left("paths").show(ctx, |ui| {
            ScrollArea::both().show(ui, |ui| {
                ui.take_available_space();

                if ui
                    .add_enabled(!is_executing, Button::new("refresh"))
                    .clicked()
                {
                    let git = Arc::clone(&self.git);
                    let paths = Arc::clone(&self.paths);
                    let ctx = ctx.clone();

                    thread::spawn(move || {
                        match git.diff_name_only() {
                            Ok(output) => {
                                let new_paths = output
                                    .stdout
                                    .split(|byte| *byte == b'\n')
                                    .map(|bytes| String::from_utf8_lossy(bytes).to_string())
                                    .collect();

                                let mut paths = paths.write().unwrap();
                                *paths = new_paths;
                            }
                            Err(error) => eprintln!("{}", error),
                        }

                        ctx.request_repaint();
                    });
                }

                for path in self.paths.read().unwrap().iter() {
                    ui.selectable_value(&mut self.selected_path, Some(path.to_owned()), path);
                }
            });
        });

        CentralPanel::default().show(ctx, |ui| {
            ui.heading("diff");
        });
    }
}
