use std::{
    collections::HashSet,
    fmt::{self, Display, Formatter},
    hash::{Hash, Hasher},
    path::PathBuf,
};

pub enum Action {
    Main(MainAction),
    Repo(RepoAction),
}

pub enum MainAction {
    Open,
    OpenRecent(PathBuf),
    Close,
}

pub enum RepoAction {
    Fetch,
    Pull,
    Push,
    Refresh,
    AddOrRestore(DiffKey),
    Commit(String),
    Switch(Branch),
    Create(String),
}

#[derive(Clone, Eq, Hash, PartialEq)]
pub struct DiffKey {
    pub path: String,
    pub area: DiffArea,
}

impl DiffKey {
    pub fn is_staged(key: &&DiffKey) -> bool {
        matches!(key.area, DiffArea::Staged)
    }

    pub fn is_not_staged(key: &&DiffKey) -> bool {
        !Self::is_staged(key)
    }
}

#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub enum DiffArea {
    Untracked,
    Unstaged,
    Staged,
}

#[derive(Default)]
pub struct Branches {
    pub current: String,
    pub other: HashSet<Branch>,
}

#[derive(Clone, Eq)]
pub struct Branch {
    pub name: String,
    pub area: BranchArea,
}

impl Display for Branch {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let Branch { name, area } = self;

        match area {
            BranchArea::Local => write!(f, "{name}"),
            BranchArea::Remote(remote) => write!(f, "{remote}/{name}"),
        }
    }
}

impl Hash for Branch {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state);
    }
}

impl PartialEq for Branch {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

#[derive(Clone, Eq, Hash, PartialEq)]
pub enum BranchArea {
    Local,
    Remote(String),
}

pub struct Log {
    pub author: String,
    pub long_date: String,
    pub short_date: String,
    pub long_hash: String,
    pub short_hash: String,
    pub subject: String,
}
