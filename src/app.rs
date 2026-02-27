use crate::{common::Action, common::DiffKey, git::Git, repo::Repo};
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
    repo: Arc<RwLock<Repo>>,
    selected_key: Option<DiffKey>,
    logs: Arc<RwLock<Vec<String>>>,
}

impl App {
    fn update(&mut self, action: Option<Action>, ctx: &Context) {
        if let Some(action) = action {
            let git = Arc::clone(&self.git);
            let repo = Arc::clone(&self.repo);
            let logs = Arc::clone(&self.logs);

            let func: Box<dyn FnOnce() + Send + 'static> = match action {
                Action::Pull => Box::new(move || {
                    let new_logs = git.pull();
                    logs.write().unwrap().extend(new_logs);
                }),
                Action::Refresh => Box::new(move || Self::refresh(&git, &repo, &logs)),
                Action::AddOrRestore(key) => Box::new(move || {
                    let new_logs = git.add_or_restore(&key);
                    logs.write().unwrap().extend(new_logs);

                    Self::refresh(&git, &repo, &logs);
                }),
            };

            Self::execute(func, ctx);
        }

        self.is_executing = Arc::strong_count(&self.git) > 1;
    }

    fn execute<F>(func: F, ctx: &Context)
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

    fn refresh(git: &Git, repo: &RwLock<Repo>, logs: &RwLock<Vec<String>>) {
        match Repo::new(git) {
            Ok(new_repo) => *repo.write().unwrap() = new_repo,
            Err(error) => logs.write().unwrap().push(error.to_string()),
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
        let mut action = None;

        TopBottomPanel::top("menu").show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui
                    .add_enabled(!self.is_executing, Button::new("refresh"))
                    .clicked()
                {
                    action = Some(Action::Refresh);
                }

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

                        for log in self.logs.read().unwrap().iter() {
                            ui.label(log);
                        }
                    });
                });
        }

        SidePanel::left("paths").show(ctx, |ui| {
            ScrollArea::both()
                .id_salt("unstaged")
                .max_height(ui.available_height() / 2.0)
                .show(ui, |ui| {
                    ui.take_available_space();

                    let repo = self.repo.read().unwrap();
                    let keys = repo.diffs.keys().filter(DiffKey::is_not_staged);
                    for key in keys {
                        if ui
                            .selectable_value(&mut self.selected_key, Some(key.clone()), &key.path)
                            .double_clicked()
                        {
                            action = Some(Action::AddOrRestore(key.clone()))
                        };
                    }
                });

            ui.separator();

            ScrollArea::both().id_salt("staged").show(ui, |ui| {
                ui.take_available_space();

                let repo = self.repo.read().unwrap();
                let keys = repo.diffs.keys().filter(DiffKey::is_staged);
                for key in keys {
                    if ui
                        .selectable_value(&mut self.selected_key, Some(key.clone()), &key.path)
                        .double_clicked()
                    {
                        action = Some(Action::AddOrRestore(key.clone()))
                    };
                }
            });
        });

        CentralPanel::default().show(ctx, |ui| {
            ScrollArea::both().show(ui, |ui| {
                ui.take_available_space();

                let repo = self.repo.read().unwrap();
                if let Some(selected_key) = &self.selected_key
                    && let Some(diff) = repo.diffs.get(selected_key)
                {
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
