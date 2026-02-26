use crate::{action::Action, git::Git};
use eframe::{
    Frame,
    egui::{
        Align, Button, CentralPanel, Color32, Context, Label, Layout, RichText, ScrollArea,
        SidePanel, TextWrapMode, TopBottomPanel,
    },
};
use std::{
    sync::{Arc, RwLock},
    thread::{self},
};

#[derive(Default)]
pub struct App {
    is_executing: bool,
    show_logs: bool,
    git: Arc<Git>,
    unstaged_paths: Arc<RwLock<Vec<String>>>,
    staged_paths: Arc<RwLock<Vec<String>>>,
    selected_path: Option<String>,
    diff: Arc<RwLock<Option<String>>>,
}

impl App {
    fn update(&mut self, action: Option<Action>, ctx: &Context) {
        if let Some(action) = action {
            let git = Arc::clone(&self.git);
            let func: Box<dyn FnOnce() + Send + 'static> = match action {
                Action::Pull => Box::new(move || match git.pull() {
                    Ok(output) => print!("{}", String::from_utf8_lossy(&output.stdout)),
                    Err(error) => eprintln!("{}", error),
                }),
                Action::RefreshUnstaged => {
                    let paths = Arc::clone(&self.unstaged_paths);

                    Box::new(move || match git.diff_name_only() {
                        Ok(names) => *paths.write().unwrap() = names,
                        Err(error) => eprintln!("{}", error),
                    })
                }
                Action::RefreshStaged => {
                    let paths = Arc::clone(&self.staged_paths);

                    Box::new(move || match git.diff_staged_name_only() {
                        Ok(names) => *paths.write().unwrap() = names,
                        Err(error) => eprintln!("{}", error),
                    })
                }
                Action::DiffUnstaged(path) => {
                    let diff = Arc::clone(&self.diff);

                    Box::new(move || match git.diff(&path) {
                        Ok(new_diff) => *diff.write().unwrap() = Some(new_diff),
                        Err(error) => eprintln!("{}", error),
                    })
                }
                Action::DiffStaged(path) => {
                    let diff = Arc::clone(&self.diff);

                    Box::new(move || match git.diff_staged(&path) {
                        Ok(new_diff) => *diff.write().unwrap() = Some(new_diff),
                        Err(error) => eprintln!("{}", error),
                    })
                }
            };

            self.execute(func, ctx);
        }

        self.is_executing = Arc::strong_count(&self.git) > 1;
    }

    fn execute<F>(&self, func: F, ctx: &Context)
    where
        F: FnOnce(),
        F: Send + 'static,
    {
        let ctx = ctx.clone();

        thread::spawn(move || {
            func();
            ctx.request_repaint();
        });
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
        let mut action = None;

        TopBottomPanel::top("menu").show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui
                    .add_enabled(!self.is_executing, Button::new("pull"))
                    .clicked()
                {
                    action = Some(Action::Pull);
                }

                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if ui.selectable_label(self.show_logs, "logs").clicked() {
                        self.show_logs = !self.show_logs;
                    }
                });
            });
        });

        if self.show_logs {
            TopBottomPanel::bottom("logs")
                .resizable(true)
                .show(ctx, |ui| {
                    ScrollArea::both().show(ui, |ui| {
                        ui.take_available_space();
                    });
                });
        }

        SidePanel::left("paths").show(ctx, |ui| {
            ScrollArea::both()
                .id_salt("unstaged")
                .max_height(ui.available_height() / 2.0)
                .show(ui, |ui| {
                    ui.take_available_space();

                    if ui
                        .add_enabled(!self.is_executing, Button::new("refresh"))
                        .clicked()
                    {
                        action = Some(Action::RefreshUnstaged);
                    }

                    for path in self.unstaged_paths.read().unwrap().iter() {
                        if ui
                            .selectable_value(&mut self.selected_path, Some(path.clone()), path)
                            .clicked()
                        {
                            action = Some(Action::DiffUnstaged(path.clone()));
                        }
                    }
                });

            ui.separator();

            ScrollArea::both().id_salt("staged").show(ui, |ui| {
                ui.take_available_space();

                if ui
                    .add_enabled(!self.is_executing, Button::new("refresh"))
                    .clicked()
                {
                    action = Some(Action::RefreshStaged);
                }

                for path in self.staged_paths.read().unwrap().iter() {
                    if ui
                        .selectable_value(
                            &mut self.selected_path,
                            Some(format!("{path}-staged")),
                            path,
                        )
                        .clicked()
                    {
                        action = Some(Action::DiffStaged(path.clone()));
                    }
                }
            });
        });

        CentralPanel::default().show(ctx, |ui| {
            ScrollArea::both().show(ui, |ui| {
                ui.take_available_space();

                if let Some(diff) = self.diff.read().unwrap().as_ref() {
                    for line in diff.lines() {
                        let colour = if line.starts_with('+') {
                            Color32::GREEN
                        } else if line.starts_with('-') {
                            Color32::RED
                        } else {
                            ui.visuals().text_color()
                        };

                        ui.add(
                            Label::new(RichText::new(line).monospace().color(colour))
                                .wrap_mode(TextWrapMode::Extend),
                        );
                    }
                }
            });
        });

        self.update(action, ctx);
    }
}
