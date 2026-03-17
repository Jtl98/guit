use crate::{
    common::{
        Action, DiffArea, DiffKey,
        FileAction::{self, Close, Init, Open, OpenRecent, RemoveRecent},
        RepoAction::{
            self, AddOrRestore, Commit, Create, Fetch, LoadLogs, Pull, Push, Refresh, Stash,
            Switch, UndoCommit,
        },
    },
    config::Config,
    execute::GitExecutor,
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
    git: Arc<Git<GitExecutor>>,
    repo: Option<Arc<RwLock<Repo>>>,
    selected_key: Option<DiffKey>,
    commit_message: Option<String>,
    branch_name: Option<String>,
    branch_search: String,
    logs_scroll_threshold: f32,
}

impl App {
    pub fn new() -> Self {
        let config = Config::new()
            .inspect_err(|error| error!("{}", error))
            .unwrap_or_default();

        Self {
            config,
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
            Close => {
                self.repo = None;
                self.selected_key = None;
                self.commit_message = None;
                self.branch_name = None;
                self.logs_scroll_threshold = 0.0;
            }
            Init => {
                let Some(dir) = FileDialog::new().pick_folder() else {
                    warn!("no folder picked");
                    return;
                };

                self.git.init(&dir);

                if let Err(error) = self.open_repo(&dir) {
                    error!("{}", error);
                }
            }
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
            RemoveRecent(repo) => {
                self.config.remove_repo(&repo);
                if let Err(error) = self.config.save() {
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
            Fetch => Box::new(move || {
                git.fetch_all();
                Self::refresh(&git, &repo);
            }),
            Pull => Box::new(move || {
                git.pull();
                Self::refresh(&git, &repo);
            }),
            Push => Box::new(move || {
                git.push();
                Self::refresh(&git, &repo);
            }),
            Refresh => Box::new(move || Self::refresh(&git, &repo)),
            AddOrRestore(DiffKey { path, area }) => Box::new(move || {
                match area {
                    DiffArea::Untracked | DiffArea::Unstaged => git.add(&path),
                    DiffArea::Staged => git.restore_staged(&path),
                }

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
                git.switch_create(&name);
                Self::refresh(&git, &repo);
            }),
            LoadLogs => Box::new(move || {
                if let Err(error) = repo.write().unwrap().load_logs(&git) {
                    error!("{}", error);
                }
            }),
            Stash => Box::new(move || {
                git.stash_push_include_untracked();
                Self::refresh(&git, &repo);
            }),
            UndoCommit => Box::new(move || {
                git.reset_soft_head_1();
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
        env::set_current_dir(&dir)?;

        let repo = Repo::new(&self.git, dir.clone())?;
        self.repo = Some(Arc::new(RwLock::new(repo)));
        self.config.add_repo(dir);
        self.config.save()?;

        Ok(())
    }

    fn refresh(git: &Git<GitExecutor>, repo: &RwLock<Repo>) {
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

            TopPanel::new(
                &self.is_executing,
                &repo.branches,
                &mut self.branch_name,
                &mut self.branch_search,
            )
            .show(ctx, &mut action);

            BottomPanel::new(
                self.is_executing,
                &mut self.show_logs,
                &repo.dir,
                &mut self.selected_key,
                &mut self.commit_message,
            )
            .show(ctx, &mut action);

            if self.show_logs {
                AppLogsPanel.show(ctx, &mut action);
            }

            PathsPanel::new(&repo.diffs, &mut self.selected_key).show(ctx, &mut action);

            if let Some(selected_key) = &self.selected_key {
                if let Some(diff) = repo.diffs.get(selected_key) {
                    DiffPanel::new(diff).show(ctx, &mut action);
                }
            } else {
                GitLogs::new(&repo.dated_logs, &mut self.logs_scroll_threshold)
                    .show(ctx, &mut action);
            }
        } else {
            WelcomePanel::new(&self.config).show(ctx, &mut action);
        }

        self.update(action, ctx);
    }
}
