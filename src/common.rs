use std::{
    cmp::{Ordering, Reverse},
    collections::{BTreeMap, BTreeSet},
    fmt::{self, Display, Formatter},
    path::PathBuf,
};

pub enum Action {
    File(FileAction),
    Repo(RepoAction),
}

pub enum FileAction {
    Close,
    Init,
    Open,
    OpenRecent(PathBuf),
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
    LoadLogs,
}

#[derive(Clone, Eq, Ord, PartialEq, PartialOrd)]
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

#[derive(Clone, Copy, Eq, Ord, PartialEq, PartialOrd)]
pub enum DiffArea {
    Untracked,
    Unstaged,
    Staged,
}

#[derive(Default)]
pub struct Branches {
    pub current: String,
    pub other: BTreeSet<Branch>,
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

impl Ord for Branch {
    fn cmp(&self, other: &Self) -> Ordering {
        self.name.cmp(&other.name)
    }
}

impl PartialEq for Branch {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl PartialOrd for Branch {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Clone, Eq, PartialEq)]
pub enum BranchArea {
    Local,
    Remote(String),
}

pub type Diffs = BTreeMap<DiffKey, String>;

pub type Logs = BTreeMap<Reverse<String>, Vec<Log>>;

pub struct Log {
    pub author: String,
    pub long_date: String,
    pub short_date: String,
    pub long_hash: String,
    pub short_hash: String,
    pub subject: String,
}
