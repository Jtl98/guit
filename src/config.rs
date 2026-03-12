use serde::{Deserialize, Serialize};
use std::{
    collections::HashSet,
    fs::{self, File},
    hash::{Hash, Hasher},
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

#[derive(Default)]
pub struct Config {
    path: PathBuf,
    repos: HashSet<RecentRepo>,
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

#[derive(Deserialize, Debug, Eq, Serialize)]
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::hash_map::DefaultHasher;

    #[test]
    fn recent_repo_new() {
        let path = PathBuf::from("/test/repo");
        let repo = RecentRepo::new(path.clone());
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        assert_eq!(repo.path, path);
        assert!(repo.opened <= now);
        assert!(repo.opened >= now.saturating_sub(1));
    }

    #[test]
    fn recent_repo_equality_same_path() {
        let path = PathBuf::from("/test/repo");
        let repo1 = RecentRepo::new(path.clone());
        let repo2 = RecentRepo::new(path);

        assert_eq!(repo1, repo2);
    }

    #[test]
    fn recent_repo_inequality_different_paths() {
        let repo1 = RecentRepo::new(PathBuf::from("/test/repo1"));
        let repo2 = RecentRepo::new(PathBuf::from("/test/repo2"));

        assert_ne!(repo1, repo2);
    }

    #[test]
    fn recent_repo_hash_equality_same_path() {
        let path = PathBuf::from("/test/repo");
        let repo1 = RecentRepo::new(path.clone());
        let repo2 = RecentRepo::new(path);
        let mut hasher1 = DefaultHasher::new();
        let mut hasher2 = DefaultHasher::new();

        repo1.hash(&mut hasher1);
        repo2.hash(&mut hasher2);

        let hash1 = hasher1.finish();
        let hash2 = hasher2.finish();

        assert_eq!(hash1, hash2);
    }

    #[test]
    fn recent_repo_hash_inequality_different_paths() {
        let repo1 = RecentRepo::new(PathBuf::from("/test/repo1"));
        let repo2 = RecentRepo::new(PathBuf::from("/test/repo2"));
        let mut hasher1 = DefaultHasher::new();
        let mut hasher2 = DefaultHasher::new();

        repo1.hash(&mut hasher1);
        repo2.hash(&mut hasher2);

        let hash1 = hasher1.finish();
        let hash2 = hasher2.finish();

        assert_ne!(hash1, hash2);
    }

    #[test]
    fn recent_repo_equality_ignores_opened() {
        let path = PathBuf::from("/test/repo");
        let mut repo1 = RecentRepo::new(path.clone());
        let mut repo2 = RecentRepo::new(path);

        repo1.opened = 1000;
        repo2.opened = 2000;

        assert_eq!(repo1, repo2);
    }
}
