use std::{
    collections::HashSet,
    hash::{Hash, Hasher},
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

#[derive(Default)]
pub struct Config {
    repos: HashSet<RecentRepo>,
}

impl Config {
    pub fn add_repo(&mut self, path: PathBuf) {
        let repo = RecentRepo::new(path);
        self.repos.replace(repo);
    }

    pub fn recent_repos(&self) -> Vec<&RecentRepo> {
        let mut repos: Vec<_> = self.repos.iter().collect();
        repos.sort_by(|a, b| b.opened.cmp(&a.opened));

        repos
    }
}

#[derive(Eq)]
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
