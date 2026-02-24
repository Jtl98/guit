use crate::{action::Action, git::Git};
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
    is_executing: bool,
    git: Arc<Git>,
    paths: Arc<RwLock<Vec<String>>>,
    selected_path: Option<String>,
    diff: Arc<RwLock<Option<String>>>,
}

impl App {
    fn update(&mut self, action: Option<Action>, ctx: &Context) {
        match action {
            Some(Action::Pull) => {
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
            Some(Action::Refresh) => {
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
            Some(Action::Diff) => {
                if let Some(selected_path) = &self.selected_path {
                    let git = Arc::clone(&self.git);
                    let selected_path = selected_path.clone();
                    let diff = Arc::clone(&self.diff);
                    let ctx = ctx.clone();

                    thread::spawn(move || {
                        match git.diff(&selected_path) {
                            Ok(output) => {
                                let new_diff = String::from_utf8_lossy(&output.stdout).to_string();
                                let mut diff = diff.write().unwrap();

                                *diff = Some(new_diff);
                            }
                            Err(error) => eprintln!("{}", error),
                        }

                        ctx.request_repaint();
                    });
                }
            }
            None => {}
        }

        self.is_executing = Arc::strong_count(&self.git) > 1;
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
        let mut action = None;

        TopBottomPanel::top("menu").show(ctx, |ui| {
            if ui
                .add_enabled(!self.is_executing, Button::new("pull"))
                .clicked()
            {
                action = Some(Action::Pull);
            }
        });

        SidePanel::left("paths").show(ctx, |ui| {
            ScrollArea::both().show(ui, |ui| {
                ui.take_available_space();

                if ui
                    .add_enabled(!self.is_executing, Button::new("refresh"))
                    .clicked()
                {
                    action = Some(Action::Refresh);
                }

                for path in self.paths.read().unwrap().iter() {
                    if ui
                        .selectable_value(&mut self.selected_path, Some(path.to_owned()), path)
                        .clicked()
                    {
                        action = Some(Action::Diff);
                    }
                }
            });
        });

        CentralPanel::default().show(ctx, |ui| {
            ScrollArea::both().show(ui, |ui| {
                if let Some(diff) = self.diff.read().unwrap().as_ref() {
                    ui.label(diff);
                }
            });
        });

        self.update(action, ctx);
    }
}
