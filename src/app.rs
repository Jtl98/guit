use crate::{
    common::{
        Action, DiffKey,
        MainAction::{self, Close, Open},
        RepoAction::{self, AddOrRestore, Commit, Fetch, Pull, Push, Refresh, Switch},
    },
    git::Git,
    log::LOGGER,
    repo::Repo,
};
use eframe::{
    Frame,
    egui::{
        Align, Button, CentralPanel, Color32, ComboBox, Context, Key, Label, Layout, RichText,
        ScrollArea, SidePanel, TextEdit, TextWrapMode, TopBottomPanel, Ui, Vec2,
    },
};
use log::{error, warn};
use rfd::FileDialog;
use std::{
    env, io,
    path::Path,
    sync::{Arc, RwLock},
    thread::{self},
};

#[derive(Default)]
pub struct App {
    is_executing: bool,
    show_logs: bool,
    git: Arc<Git>,
    repo: Option<Arc<RwLock<Repo>>>,
    selected_key: Option<DiffKey>,
    commit_message: String,
}

impl App {
    fn update(&mut self, action: Option<Action>, ctx: &Context) {
        if let Some(action) = action {
            match action {
                Action::Main(action) => self.execute_main_action(action),
                Action::Repo(action) => self.execute_repo_action(action, ctx),
            }
        }

        self.is_executing = Arc::strong_count(&self.git) > 1;
    }

    fn execute_main_action(&mut self, action: MainAction) {
        match action {
            Open => {
                let Some(dir) = FileDialog::new().pick_folder() else {
                    warn!("no folder picked");
                    return;
                };

                match self.open_repo(dir) {
                    Ok(repo) => self.repo = Some(Arc::new(RwLock::new(repo))),
                    Err(error) => error!("{}", error),
                }
            }
            Close => {
                self.repo = None;
            }
        }
    }

    fn execute_repo_action(&self, action: RepoAction, ctx: &Context) {
        let Some(ref repo) = self.repo else {
            return;
        };

        let git = Arc::clone(&self.git);
        let repo = Arc::clone(repo);

        let func: Box<dyn FnOnce() + Send + 'static> = match action {
            Fetch => Box::new(move || git.fetch_all()),
            Pull => Box::new(move || git.pull()),
            Push => Box::new(move || git.push()),
            Refresh => Box::new(move || Self::refresh(&git, &repo)),
            AddOrRestore(key) => Box::new(move || {
                git.add_or_restore(&key);
                Self::refresh(&git, &repo);
            }),
            Commit(message) => Box::new(move || {
                git.commit(&message);
                Self::refresh(&git, &repo);
            }),
            Switch(branch) => Box::new(move || {
                git.switch(&branch);
                Self::refresh(&git, &repo);
            }),
        };

        Self::execute(func, ctx);
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

    fn open_repo<P: AsRef<Path>>(&self, dir: P) -> io::Result<Repo> {
        let dir = self.git.rev_parse_show_toplevel(dir)?;
        env::set_current_dir(&dir)?;
        Repo::new(&self.git, dir)
    }

    fn refresh(git: &Git, repo: &RwLock<Repo>) {
        let dir = repo.read().unwrap().dir.clone();
        match Repo::new(git, dir) {
            Ok(new_repo) => *repo.write().unwrap() = new_repo,
            Err(error) => error!("{}", error),
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
        let mut action = None;

        let Some(ref repo) = self.repo else {
            CentralPanel::default().show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.spacing_mut().button_padding = Vec2::new(32.0, 16.0);
                    ui.add_space(ui.available_height() / 3.0);

                    if ui.button(RichText::new("open").size(32.0)).clicked() {
                        action = Some(Action::Main(Open));
                    }
                });
            });

            self.update(action, ctx);

            return;
        };

        TopBottomPanel::top("top_menu").show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui
                    .add_enabled(!self.is_executing, Button::new("refresh"))
                    .clicked()
                {
                    action = Some(Action::Repo(Refresh));
                }

                if ui
                    .add_enabled(!self.is_executing, Button::new("fetch"))
                    .clicked()
                {
                    action = Some(Action::Repo(Fetch));
                }

                if ui
                    .add_enabled(!self.is_executing, Button::new("pull"))
                    .clicked()
                {
                    action = Some(Action::Repo(Pull));
                }

                if ui
                    .add_enabled(!self.is_executing, Button::new("push"))
                    .clicked()
                {
                    action = Some(Action::Repo(Push));
                }

                let repo = repo.read().unwrap();

                ComboBox::from_id_salt("branches")
                    .selected_text(&repo.branches.current)
                    .show_ui(ui, |ui| {
                        for branch in &repo.branches.other {
                            if ui.selectable_label(false, branch.to_string()).clicked() {
                                action = Some(Action::Repo(Switch(branch.clone())));
                            }
                        }
                    });

                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if ui.button("close").clicked() {
                        action = Some(Action::Main(Close));
                    }

                    if self.is_executing {
                        ui.spinner();
                    }
                });
            });
        });

        TopBottomPanel::bottom("bottom_menu").show(ctx, |ui| {
            ui.horizontal(|ui| {
                let commit_message_provided = !self.commit_message.trim().is_empty();
                let text = ui.add_enabled(
                    !self.is_executing,
                    TextEdit::singleline(&mut self.commit_message),
                );
                let button = ui.add_enabled(
                    !self.is_executing && commit_message_provided,
                    Button::new("commit"),
                );

                if commit_message_provided
                    && (button.clicked()
                        || (text.lost_focus() && ui.input(|i| i.key_pressed(Key::Enter))))
                {
                    action = Some(Action::Repo(Commit(self.commit_message.clone())));
                    self.commit_message.clear();
                }

                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if ui.selectable_label(self.show_logs, "logs").clicked() {
                        self.show_logs = !self.show_logs;
                    }

                    let dir = &repo.read().unwrap().dir;
                    ui.label(dir);
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

                        let repo = repo.read().unwrap();
                        let keys = repo.diffs.keys().filter(filter);
                        for key in keys {
                            let checked = self.selected_key.as_ref() == Some(key);
                            let response = ui.selectable_label(checked, &key.path);

                            if response.double_clicked() {
                                action = Some(Action::Repo(AddOrRestore(key.clone())));
                                self.selected_key = None;
                            } else if response.clicked() {
                                self.selected_key = if checked { None } else { Some(key.clone()) }
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

                let repo = repo.read().unwrap();
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
