use serde::{Deserialize, Serialize};
use std::{
    collections::HashSet,
    fs::File,
    hash::{Hash, Hasher},
    io,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

type RecentRepos = HashSet<RecentRepo>;

#[derive(Default)]
pub struct Config {
    repos: RecentRepos,
}

impl Config {
    const DIR: &str = env!("CARGO_MANIFEST_DIR");
    const FILE: &str = "config.json";

    pub fn new() -> io::Result<Self> {
        let mut new = Self::default();
        new.load()?;

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

    pub fn load(&mut self) -> io::Result<()> {
        let file = self.open_file()?;
        let repos: RecentRepos = serde_json::from_reader(file)?;

        self.repos = repos;
        Ok(())
    }

    pub fn save(&self) -> io::Result<()> {
        let file = self.open_file()?;

        serde_json::to_writer(file, &self.repos)?;
        Ok(())
    }

    fn open_file(&self) -> io::Result<File> {
        let path = format!("{}/{}", Self::DIR, Self::FILE);
        File::options()
            .create(true)
            .write(true)
            .read(true)
            .open(path)
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
