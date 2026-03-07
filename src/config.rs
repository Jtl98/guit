use serde::{Deserialize, Serialize};
use std::{
    collections::HashSet,
    fs::{self, File},
    hash::{Hash, Hasher},
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

type RecentRepos = HashSet<RecentRepo>;

#[derive(Default)]
pub struct Config {
    path: PathBuf,
    repos: RecentRepos,
}

impl Config {
    pub fn new() -> anyhow::Result<Self> {
        let dir = dirs::config_dir().unwrap_or_default().join("guit");
        fs::create_dir_all(&dir)?;

        let mut new = Self {
            path: dir.join("config.json"),
            ..Default::default()
        };

        if fs::exists(&new.path)? {
            new.load()?;
        } else {
            new.save()?;
        }

        Ok(new)
    }

    pub fn add_repo(&mut self, path: PathBuf) {
        let repo = RecentRepo::new(path);
        self.repos.replace(repo);
    }

    pub fn recent_repos(&self) -> Vec<&RecentRepo> {
        let mut repos: Vec<_> = self.repos.iter().collect();
        repos.sort_by(|a, b| b.opened.cmp(&a.opened));

        repos
    }

    pub fn save(&self) -> anyhow::Result<()> {
        let file = self.open_file()?;
        serde_json::to_writer(file, &self.repos)?;
        Ok(())
    }

    fn load(&mut self) -> anyhow::Result<()> {
        let file = self.open_file()?;
        self.repos = serde_json::from_reader(file)?;
        Ok(())
    }

    fn open_file(&self) -> anyhow::Result<File> {
        let file = File::options()
            .create(true)
            .truncate(false)
            .write(true)
            .read(true)
            .open(&self.path)?;

        Ok(file)
    }
}

#[derive(Deserialize, Eq, Serialize)]
pub struct RecentRepo {
    pub path: PathBuf,
    opened: u64,
}

impl RecentRepo {
    fn new(path: PathBuf) -> Self {
        let opened = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        Self { path, opened }
    }
}

impl Hash for RecentRepo {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.path.hash(state);
    }
}

impl PartialEq for RecentRepo {
    fn eq(&self, other: &Self) -> bool {
        self.path == other.path
    }
}
