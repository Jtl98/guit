use crate::{
    common::{
        Action, DiffKey,
        FileAction::{self, Close, Init, Open, OpenRecent},
        RepoAction::{self, AddOrRestore, Commit, Create, Fetch, Pull, Push, Refresh, Switch},
    },
    config::Config,
    git::Git,
    panels::{
        Show, app_logs::AppLogsPanel, bottom::BottomPanel, diff::DiffPanel, git_logs::GitLogs,
        paths::PathsPanel, top::TopPanel, welcome::WelcomePanel,
    },
    repo::Repo,
};
use eframe::{Frame, egui::Context};
use log::{error, warn};
use rfd::FileDialog;
use std::{
    env,
    path::Path,
    sync::{Arc, RwLock},
    thread::{self},
};

#[derive(Default)]
pub struct App {
    config: Config,
    is_executing: bool,
    show_logs: bool,
    git: Arc<Git>,
    repo: Option<Arc<RwLock<Repo>>>,
    selected_key: Option<DiffKey>,
    commit_message: String,
    branch_name: String,
    init_name: String,
}

impl App {
    pub fn new() -> Self {
        let config = Config::new()
            .inspect_err(|error| error!("{}", error))
            .unwrap_or_default();

        Self {
            config,
            init_name: "main".to_owned(),
            ..Default::default()
        }
    }

    fn update(&mut self, action: Option<Action>, ctx: &Context) {
        if let Some(action) = action {
            match action {
                Action::File(action) => self.execute_file_action(action),
                Action::Repo(action) => self.execute_repo_action(action, ctx),
            }
        }

        self.is_executing = Arc::strong_count(&self.git) > 1;
    }

    fn execute_file_action(&mut self, action: FileAction) {
        match action {
            Open => {
                let Some(dir) = FileDialog::new().pick_folder() else {
                    warn!("no folder picked");
                    return;
                };

                if let Err(error) = self.open_repo(&dir) {
                    error!("{}", error);
                }
            }
            OpenRecent(path) => {
                if let Err(error) = self.open_repo(&path) {
                    error!("{}", error);
                }
            }
            Close => {
                self.repo = None;
            }
            Init(name) => {
                let Some(dir) = FileDialog::new().pick_folder() else {
                    warn!("no folder picked");
                    return;
                };

                self.git.init(&name, &dir);

                if let Err(error) = self.open_repo(&dir) {
                    error!("{}", error);
                }
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
            Create(name) => Box::new(move || {
                git.branch_create(&name);
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

    fn open_repo(&mut self, dir: &Path) -> anyhow::Result<()> {
        let dir = self.git.rev_parse_show_toplevel(dir)?;
        let repo = Repo::new(&self.git, dir.clone())?;
        env::set_current_dir(&dir)?;

        self.config.add_repo(dir);
        self.repo = Some(Arc::new(RwLock::new(repo)));
        self.config.save()?;

        Ok(())
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

        if let Some(repo) = &self.repo {
            let repo = repo.read().unwrap();

            TopPanel::new(&self.is_executing, &repo, &mut self.branch_name).show(ctx, &mut action);

            BottomPanel::new(
                self.is_executing,
                &mut self.show_logs,
                &repo,
                &mut self.selected_key,
                &mut self.commit_message,
            )
            .show(ctx, &mut action);

            if self.show_logs {
                AppLogsPanel.show(ctx, &mut action);
            }

            PathsPanel::new(&repo, &mut self.selected_key).show(ctx, &mut action);

            if let Some(selected_key) = &self.selected_key {
                if let Some(diff) = repo.diffs.get(selected_key) {
                    DiffPanel::new(diff).show(ctx, &mut action);
                }
            } else {
                GitLogs::new(&repo.logs).show(ctx, &mut action);
            }
        } else {
            WelcomePanel::new(&self.config, &mut self.init_name).show(ctx, &mut action);
        }

        self.update(action, ctx);
    }
}
