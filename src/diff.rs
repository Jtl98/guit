#[derive(Clone, Eq, Hash, PartialEq)]
pub struct DiffKey {
    pub path: String,
    pub area: DiffArea,
}

impl DiffKey {
    pub fn is_unstaged(key: &&DiffKey) -> bool {
        matches!(key.area, DiffArea::Unstaged)
    }

    pub fn is_staged(key: &&DiffKey) -> bool {
        matches!(key.area, DiffArea::Staged)
    }
}

#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub enum DiffArea {
    Unstaged,
    Staged,
}
