pub enum Action {
    Pull,
    Push,
    Refresh,
    AddOrRestore(DiffKey),
    Commit(String),
    Switch(String),
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
    pub other: Vec<String>,
}
