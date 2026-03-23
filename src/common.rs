use crate::config::RecentRepo;
use anyhow::anyhow;
use std::{
    borrow::Cow,
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
    RemoveRecent(RecentRepo),
}

pub enum RepoAction {
    Fetch,
    Pull,
    Push,
    Refresh,
    AddOrRestore(DiffKey),
    AddAll,
    RestoreAll,
    Commit(String, Option<String>),
    Switch(Branch),
    Create(String),
    LoadLogs(f32),
    StashPop,
    StashPush,
    UndoCommit,
}

pub struct Diff {
    pub content: String,
    pub numstat: DiffNumstat,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
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

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum DiffArea {
    Untracked,
    Unstaged,
    Staged,
}

pub struct DiffNumstat {
    pub additions: String,
    pub deletions: String,
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

pub type Diffs = BTreeMap<DiffKey, Diff>;

pub type DatedLogs = BTreeMap<Reverse<String>, Vec<Log>>;

pub struct Log {
    pub author: String,
    pub long_date: String,
    pub short_date: String,
    pub long_hash: String,
    pub short_hash: String,
    pub subject: String,
    pub body: Option<String>,
}

pub fn split_by_byte(bytes: &[u8], byte: u8) -> impl Iterator<Item = &[u8]> {
    bytes
        .split(move |&b| b == byte)
        .map(<[u8]>::trim_ascii)
        .filter(|b| !b.is_empty())
}

pub fn split_by_byte_to_string(bytes: &[u8], byte: u8) -> impl Iterator<Item = String> {
    split_by_byte(bytes, byte)
        .map(String::from_utf8_lossy)
        .map(Cow::into_owned)
}

pub fn split_by_newline<B: FromIterator<String>>(bytes: &[u8]) -> B {
    String::from_utf8_lossy(bytes)
        .lines()
        .filter(|line| !line.is_empty())
        .map(str::to_string)
        .collect()
}

pub fn split_whitespace_take<const N: usize>(bytes: &[u8]) -> anyhow::Result<[String; N]> {
    String::from_utf8_lossy(bytes)
        .split_whitespace()
        .take(N)
        .map(str::to_string)
        .collect::<Vec<String>>()
        .try_into()
        .map_err(|v: Vec<String>| anyhow!("expected {N} parts, received {}", v.len()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_by_byte_basic() {
        let bytes = b"line1,line2,line3";
        let result: Vec<&[u8]> = split_by_byte(bytes, b',').collect();
        assert_eq!(result, vec![b"line1", b"line2", b"line3"]);
    }

    #[test]
    fn split_by_byte_multiple_delimiters() {
        let bytes = b"a,,b,,c";
        let result: Vec<&[u8]> = split_by_byte(bytes, b',').collect();
        assert_eq!(result, vec![b"a", b"b", b"c"]);
    }

    #[test]
    fn split_by_byte_empty_input() {
        let bytes = b"";
        let result: Vec<&[u8]> = split_by_byte(bytes, b',').collect();
        assert!(result.is_empty());
    }

    #[test]
    fn split_by_byte_only_delimiter() {
        let bytes = b",,,";
        let result: Vec<&[u8]> = split_by_byte(bytes, b',').collect();
        assert!(result.is_empty());
    }

    #[test]
    fn split_by_byte_leading_delimiter() {
        let bytes = b",line1,line2";
        let result: Vec<&[u8]> = split_by_byte(bytes, b',').collect();
        assert_eq!(result, vec![b"line1", b"line2"]);
    }

    #[test]
    fn split_by_byte_trailing_delimiter() {
        let bytes = b"line1,line2,";
        let result: Vec<&[u8]> = split_by_byte(bytes, b',').collect();
        assert_eq!(result, vec![b"line1", b"line2"]);
    }

    #[test]
    fn split_by_byte_single_element() {
        let bytes = b"line1";
        let result: Vec<&[u8]> = split_by_byte(bytes, b',').collect();
        assert_eq!(result, vec![b"line1"]);
    }

    #[test]
    fn split_by_byte_space_delimiter() {
        let bytes = b"word1 word2 word3";
        let result: Vec<&[u8]> = split_by_byte(bytes, b' ').collect();
        assert_eq!(result, vec![b"word1", b"word2", b"word3"]);
    }

    #[test]
    fn split_by_byte_trims_whitespace() {
        let bytes = b"  line1  ,  line2  ";
        let result: Vec<&[u8]> = split_by_byte(bytes, b',').collect();
        assert_eq!(result, vec![b"line1", b"line2"]);
    }

    #[test]
    fn split_by_byte_unicode() {
        let bytes = "línea1|línea2|日本語".as_bytes();
        let result: Vec<&[u8]> = split_by_byte(bytes, b'|').collect();
        assert_eq!(
            result,
            vec![
                "línea1".as_bytes(),
                "línea2".as_bytes(),
                "日本語".as_bytes()
            ]
        );
    }

    #[test]
    fn split_by_byte_to_string_basic() {
        let bytes = b"line1,line2,line3";
        let result: Vec<String> = split_by_byte_to_string(bytes, b',').collect();
        assert_eq!(result, vec!["line1", "line2", "line3"]);
    }

    #[test]
    fn split_by_byte_to_string_empty_input() {
        let bytes = b"";
        let result: Vec<String> = split_by_byte_to_string(bytes, b',').collect();
        assert!(result.is_empty());
    }

    #[test]
    fn split_by_byte_to_string_single_element() {
        let bytes = b"line1";
        let result: Vec<String> = split_by_byte_to_string(bytes, b',').collect();
        assert_eq!(result, vec!["line1"]);
    }

    #[test]
    fn split_by_byte_to_string_multiple_delimiters() {
        let bytes = b"a,,b,,c";
        let result: Vec<String> = split_by_byte_to_string(bytes, b',').collect();
        assert_eq!(result, vec!["a", "b", "c"]);
    }

    #[test]
    fn split_by_byte_to_string_newline() {
        let bytes = b"line1\nline2\nline3";
        let result: Vec<String> = split_by_byte_to_string(bytes, b'\n').collect();
        assert_eq!(result, vec!["line1", "line2", "line3"]);
    }

    #[test]
    fn split_by_byte_to_string_trims_whitespace() {
        let bytes = b"  line1  ,  line2  ";
        let result: Vec<String> = split_by_byte_to_string(bytes, b',').collect();
        assert_eq!(result, vec!["line1", "line2"]);
    }

    #[test]
    fn split_by_byte_to_string_unicode() {
        let bytes = "línea1|línea2|日本語".as_bytes();
        let result: Vec<String> = split_by_byte_to_string(bytes, b'|').collect();
        assert_eq!(result, vec!["línea1", "línea2", "日本語"]);
    }

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

    #[test]
    fn split_whitespace_take_basic() {
        let bytes = b"word1 word2 word3";
        let result = split_whitespace_take::<3>(bytes).unwrap();
        assert_eq!(result, ["word1", "word2", "word3"]);
    }

    #[test]
    fn split_whitespace_take_multiple_spaces() {
        let bytes = b"word1  word2   word3";
        let result = split_whitespace_take::<3>(bytes).unwrap();
        assert_eq!(result, ["word1", "word2", "word3"]);
    }

    #[test]
    fn split_whitespace_take_empty_bytes() {
        let bytes = b"";
        let result = split_whitespace_take::<0>(bytes).unwrap();
        assert_eq!(result, <[String; 0]>::default());
    }

    #[test]
    fn split_whitespace_take_only_whitespace() {
        let bytes = b"   \t\n  ";
        let result = split_whitespace_take::<0>(bytes).unwrap();
        assert_eq!(result, <[String; 0]>::default());
    }

    #[test]
    fn split_whitespace_take_single_word() {
        let bytes = b"word1";
        let result = split_whitespace_take::<1>(bytes).unwrap();
        assert_eq!(result, ["word1"]);
    }

    #[test]
    fn split_whitespace_take_leading_whitespace() {
        let bytes = b"   word1 word2";
        let result = split_whitespace_take::<2>(bytes).unwrap();
        assert_eq!(result, ["word1", "word2"]);
    }

    #[test]
    fn split_whitespace_take_trailing_whitespace() {
        let bytes = b"word1 word2   ";
        let result = split_whitespace_take::<2>(bytes).unwrap();
        assert_eq!(result, ["word1", "word2"]);
    }

    #[test]
    fn split_whitespace_take_unicode() {
        let bytes = "función éxito 日本語".as_bytes();
        let result = split_whitespace_take::<3>(bytes).unwrap();
        assert_eq!(result, ["función", "éxito", "日本語"]);
    }

    #[test]
    fn split_whitespace_take_mixed_whitespace() {
        let bytes = b"word1\tword2\nword3  word4";
        let result = split_whitespace_take::<4>(bytes).unwrap();
        assert_eq!(result, ["word1", "word2", "word3", "word4"]);
    }

    #[test]
    fn split_whitespace_take_n_zero() {
        let bytes = b"word1 word2 word3";
        let result = split_whitespace_take::<0>(bytes).unwrap();
        assert_eq!(result, <[String; 0]>::default());
    }

    #[test]
    fn split_whitespace_take_n_one() {
        let bytes = b"word1 word2 word3";
        let result = split_whitespace_take::<1>(bytes).unwrap();
        assert_eq!(result, ["word1"]);
    }

    #[test]
    fn split_whitespace_take_n_two() {
        let bytes = b"word1 word2 word3 word4";
        let result = split_whitespace_take::<2>(bytes).unwrap();
        assert_eq!(result, ["word1", "word2"]);
    }

    #[test]
    fn split_whitespace_take_n_exact() {
        let bytes = b"word1 word2 word3";
        let result = split_whitespace_take::<3>(bytes).unwrap();
        assert_eq!(result, ["word1", "word2", "word3"]);
    }

    #[test]
    fn split_whitespace_take_n_more_than_available() {
        let bytes = b"word1 word2";
        let result: Result<[String; 5], _> = split_whitespace_take(bytes);
        assert!(result.is_err());
    }

    #[test]
    fn split_whitespace_take_n_from_empty() {
        let bytes = b"";
        let result: Result<[String; 3], _> = split_whitespace_take(bytes);
        assert!(result.is_err());
    }

    #[test]
    fn split_whitespace_take_n_unicode() {
        let bytes = "función éxito 日本語".as_bytes();
        let result = split_whitespace_take::<2>(bytes).unwrap();
        assert_eq!(result, ["función", "éxito"]);
    }

    #[test]
    fn split_whitespace_take_n_mixed_whitespace_partial() {
        let bytes = b"word1\tword2\nword3  word4\tword5";
        let result = split_whitespace_take::<3>(bytes).unwrap();
        assert_eq!(result, ["word1", "word2", "word3"]);
    }

    #[test]
    fn split_whitespace_take_n_leading_whitespace() {
        let bytes = b"   word1 word2 word3";
        let result = split_whitespace_take::<2>(bytes).unwrap();
        assert_eq!(result, ["word1", "word2"]);
    }

    #[test]
    fn split_whitespace_take_n_trailing_whitespace() {
        let bytes = b"word1 word2 word3   ";
        let result = split_whitespace_take::<2>(bytes).unwrap();
        assert_eq!(result, ["word1", "word2"]);
    }
}
