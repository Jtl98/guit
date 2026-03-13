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

pub type DatedLogs = BTreeMap<Reverse<String>, Vec<Log>>;

pub struct Log {
    pub author: String,
    pub long_date: String,
    pub short_date: String,
    pub long_hash: String,
    pub short_hash: String,
    pub subject: String,
}

pub fn split_by_newline<B: FromIterator<String>>(bytes: &[u8]) -> B {
    String::from_utf8_lossy(bytes)
        .lines()
        .filter(|line| !line.is_empty())
        .map(str::to_string)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_by_newline_basic() {
        let bytes = b"line1\nline2\nline3";
        let result: Vec<String> = split_by_newline(bytes);

        assert_eq!(result, vec!["line1", "line2", "line3"]);
    }

    #[test]
    fn split_by_newline_empty_lines() {
        let bytes = b"line1\n\nline2\n\n\nline3";
        let result: Vec<String> = split_by_newline(bytes);

        assert_eq!(result, vec!["line1", "line2", "line3"]);
    }

    #[test]
    fn split_by_newline_empty_bytes() {
        let bytes = b"";
        let result: Vec<String> = split_by_newline(bytes);

        assert!(result.is_empty());
    }

    #[test]
    fn split_by_newline_only_newlines() {
        let bytes = b"\n\n\n";
        let result: Vec<String> = split_by_newline(bytes);

        assert!(result.is_empty());
    }

    #[test]
    fn split_by_newline_single_line() {
        let bytes = b"line1";
        let result: Vec<String> = split_by_newline(bytes);

        assert_eq!(result, vec!["line1"]);
    }

    #[test]
    fn split_by_newline_trailing_newline() {
        let bytes = b"line1\nline2\n";
        let result: Vec<String> = split_by_newline(bytes);

        assert_eq!(result, vec!["line1", "line2"]);
    }

    #[test]
    fn split_by_newline_leading_newline() {
        let bytes = b"\nline1\nline2";
        let result: Vec<String> = split_by_newline(bytes);

        assert_eq!(result, vec!["line1", "line2"]);
    }

    #[test]
    fn split_by_newline_unicode() {
        let bytes = " línea1\n línea2\n日本語".as_bytes();
        let result: Vec<String> = split_by_newline(bytes);

        assert_eq!(result, vec![" línea1", " línea2", "日本語"]);
    }

    #[test]
    fn split_by_newline_multiple_trailing_newlines() {
        let bytes = b"line1\n\n\n";
        let result: Vec<String> = split_by_newline(bytes);

        assert_eq!(result, vec!["line1"]);
    }

    #[test]
    fn split_by_newline_multiple_leading_newlines() {
        let bytes = b"\n\n\nline1";
        let result: Vec<String> = split_by_newline(bytes);

        assert_eq!(result, vec!["line1"]);
    }

    #[test]
    fn split_by_newline_carriage_return() {
        let bytes = b"line1\r\nline2\r\nline3";
        let result: Vec<String> = split_by_newline(bytes);

        assert_eq!(result, vec!["line1", "line2", "line3"]);
    }
}
