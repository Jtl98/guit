use crate::{
    common::{Action, DiffKey},
    git::Git,
    log::LOGGER,
    repo::Repo,
};
use eframe::{
    Frame,
    egui::{
        Align, Button, CentralPanel, Color32, ComboBox, Context, Label, Layout, RichText,
        ScrollArea, SidePanel, TextWrapMode, TopBottomPanel, Ui,
    },
};
use log::{error, warn};
use rfd::FileDialog;
use std::{
    env,
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
    commit_message: String,
}

impl App {
    fn update(&mut self, action: Option<Action>, ctx: &Context) {
        if let Some(action) = action {
            let git = Arc::clone(&self.git);
            let repo = Arc::clone(&self.repo);

            let func: Box<dyn FnOnce() + Send + 'static> = match action {
                Action::Open => {
                    let Some(dir) = FileDialog::new().pick_folder() else {
                        warn!("no folder picked");
                        return;
                    };

                    Box::new(move || match git.rev_parse_show_toplevel(dir) {
                        Ok(dir) => match env::set_current_dir(dir) {
                            Ok(_) => Self::refresh(&git, &repo),
                            Err(error) => error!("{}", error),
                        },
                        Err(error) => error!("{}", error),
                    })
                }
                Action::Fetch => Box::new(move || git.fetch_all()),
                Action::Pull => Box::new(move || git.pull()),
                Action::Push => Box::new(move || git.push()),
                Action::Refresh => Box::new(move || Self::refresh(&git, &repo)),
                Action::AddOrRestore(key) => Box::new(move || {
                    git.add_or_restore(&key);
                    Self::refresh(&git, &repo);
                }),
                Action::Commit(message) => Box::new(move || {
                    git.commit(&message);
                    Self::refresh(&git, &repo);
                }),
                Action::Switch(branch) => Box::new(move || {
                    git.switch(&branch);
                    Self::refresh(&git, &repo);
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

    fn refresh(git: &Git, repo: &RwLock<Repo>) {
        match Repo::new(git) {
            Ok(new_repo) => *repo.write().unwrap() = new_repo,
            Err(error) => error!("{}", error),
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
        let mut action = None;

        TopBottomPanel::top("top_menu").show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui
                    .add_enabled(!self.is_executing, Button::new("open"))
                    .clicked()
                {
                    action = Some(Action::Open);
                }

                if ui
                    .add_enabled(!self.is_executing, Button::new("refresh"))
                    .clicked()
                {
                    action = Some(Action::Refresh);
                }

                if ui
                    .add_enabled(!self.is_executing, Button::new("fetch"))
                    .clicked()
                {
                    action = Some(Action::Fetch);
                }

                if ui
                    .add_enabled(!self.is_executing, Button::new("pull"))
                    .clicked()
                {
                    action = Some(Action::Pull);
                }

                if ui
                    .add_enabled(!self.is_executing, Button::new("push"))
                    .clicked()
                {
                    action = Some(Action::Push);
                }

                let repo = self.repo.read().unwrap();

                ComboBox::from_id_salt("branches")
                    .selected_text(&repo.branches.current)
                    .show_ui(ui, |ui| {
                        for branch in &repo.branches.other {
                            if ui.selectable_label(false, branch.to_string()).clicked() {
                                action = Some(Action::Switch(branch.clone()));
                            }
                        }
                    });

                if self.is_executing {
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        ui.spinner();
                    });
                }
            });
        });

        TopBottomPanel::bottom("bottom_menu").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.text_edit_singleline(&mut self.commit_message);
                if ui
                    .add_enabled(
                        !self.is_executing && !self.commit_message.is_empty(),
                        Button::new("commit"),
                    )
                    .clicked()
                {
                    action = Some(Action::Commit(self.commit_message.clone()));
                    self.commit_message.clear();
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
                    ui.with_layout(Layout::right_to_left(Align::Min), |ui| {
                        if ui.button("clear").clicked() {
                            LOGGER.clear();
                        }
                    });

                    ScrollArea::both().show(ui, |ui| {
                        ui.take_available_space();

                        for entry in LOGGER.read().iter() {
                            let colour = match entry.level {
                                log::Level::Error => Color32::RED,
                                log::Level::Warn => Color32::YELLOW,
                                log::Level::Info => Color32::WHITE,
                                log::Level::Debug | log::Level::Trace => ui.visuals().text_color(),
                            };

                            ui.add(
                                Label::new(RichText::new(entry).monospace().color(colour))
                                    .wrap_mode(TextWrapMode::Extend),
                            );
                        }
                    });
                });
        }

        SidePanel::left("paths").show(ctx, |ui| {
            let mut show_keys = |ui: &mut Ui, id: &str, filter: fn(&&DiffKey) -> bool| {
                ScrollArea::both()
                    .id_salt(id)
                    .max_height(ui.available_height() / 2.0)
                    .show(ui, |ui| {
                        ui.take_available_space();

                        let repo = self.repo.read().unwrap();
                        let keys = repo.diffs.keys().filter(filter);
                        for key in keys {
                            let response = ui.selectable_label(
                                self.selected_key.as_ref() == Some(key),
                                &key.path,
                            );

                            if response.double_clicked() {
                                self.selected_key = None;
                                action = Some(Action::AddOrRestore(key.clone()));
                            } else if response.clicked() {
                                self.selected_key = if self.selected_key.is_none() {
                                    Some(key.clone())
                                } else {
                                    None
                                }
                            }
                        }
                    });
            };

            show_keys(ui, "unstaged", DiffKey::is_not_staged);
            ui.separator();
            show_keys(ui, "staged", DiffKey::is_staged);
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
