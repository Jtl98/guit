use std::collections::BTreeSet;

use crate::common::{self, Branch, BranchArea, DiffArea, DiffKey, DiffNumstat, Hunk, Log};

#[derive(Default)]
pub struct GitParser;

impl GitParser {
    const HUNK_HEADER_PREFIX: &str = "@@";
    const NULL: u8 = b'\x00';
    const US: u8 = b'\x1f';

    pub fn parse_diff_keys(&self, bytes: &[u8], area: DiffArea) -> Vec<DiffKey> {
        common::split_by_newline::<Vec<String>>(bytes)
            .into_iter()
            .map(|path| DiffKey { path, area })
            .collect()
    }

    pub fn parse_hunks(&self, bytes: &[u8]) -> Vec<Hunk> {
        let diff = String::from_utf8_lossy(bytes);
        let lines = diff.lines().skip(4);
        let mut hunks = Vec::new();
        let mut current_hunk = None;

        for line in lines {
            if line.starts_with(Self::HUNK_HEADER_PREFIX) {
                if let Some(hunk) = current_hunk {
                    hunks.push(hunk);
                }

                current_hunk = Some(Hunk {
                    header: line.to_owned(),
                    lines: Vec::new(),
                });
            } else if let Some(ref mut hunk) = current_hunk {
                hunk.lines.push(line.to_owned());
            }
        }

        if let Some(hunk) = current_hunk {
            hunks.push(hunk);
        }

        hunks
    }

    pub fn parse_local_branches(&self, bytes: &[u8]) -> (String, BTreeSet<Branch>) {
        let mut current = String::new();
        let mut other = BTreeSet::new();

        let branches = common::split_by_newline::<Vec<String>>(bytes);
        for branch in branches {
            let name = branch[2..].to_owned();

            if branch.starts_with("* ") {
                current = name;
            } else {
                other.insert(Branch {
                    name,
                    area: BranchArea::Local,
                });
            }
        }

        (current, other)
    }

    pub fn parse_logs(&self, bytes: &[u8]) -> Vec<Log> {
        common::split_by_byte(bytes, Self::NULL)
            .filter_map(|log| {
                let mut parts = common::split_by_byte_to_string(log, Self::US);

                Some(Log {
                    author: parts.next()?,
                    long_date: parts.next()?,
                    short_date: parts.next()?,
                    long_hash: parts.next()?,
                    short_hash: parts.next()?,
                    subject: parts.next()?,
                    body: parts.next(),
                })
            })
            .collect()
    }

    pub fn parse_numstat(&self, bytes: &[u8]) -> anyhow::Result<DiffNumstat> {
        let [additions, deletions] = common::split_whitespace_take::<2>(bytes)?;

        Ok(DiffNumstat {
            additions,
            deletions,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_parser() -> GitParser {
        GitParser::default()
    }

    #[test]
    fn parse_diff_keys_empty_bytes() {
        let parser = create_parser();
        let bytes = b"";

        for area in [DiffArea::Untracked, DiffArea::Unstaged, DiffArea::Staged] {
            let result = parser.parse_diff_keys(bytes, area);

            assert!(result.is_empty());
        }
    }

    #[test]
    fn parse_diff_keys_single_path() {
        let parser = create_parser();
        let bytes = b"src/main.rs\n";

        for area in [DiffArea::Untracked, DiffArea::Unstaged, DiffArea::Staged] {
            let result = parser.parse_diff_keys(bytes, area);

            let expected = vec![DiffKey {
                path: "src/main.rs".to_string(),
                area,
            }];
            assert_eq!(result, expected);
        }
    }

    #[test]
    fn parse_diff_keys_multiple_paths() {
        let parser = create_parser();
        let bytes = b"src/main.rs\nsrc/lib.rs\nCargo.toml\n";
        for area in [DiffArea::Untracked, DiffArea::Unstaged, DiffArea::Staged] {
            let result = parser.parse_diff_keys(bytes, area);

            let expected = vec![
                DiffKey {
                    path: "src/main.rs".to_string(),
                    area,
                },
                DiffKey {
                    path: "src/lib.rs".to_string(),
                    area,
                },
                DiffKey {
                    path: "Cargo.toml".to_string(),
                    area,
                },
            ];
            assert_eq!(result, expected);
        }
    }

    #[test]
    fn parse_diff_keys_path_with_spaces() {
        let parser = create_parser();
        let bytes = b"my file.txt\n";
        for area in [DiffArea::Untracked, DiffArea::Unstaged, DiffArea::Staged] {
            let result = parser.parse_diff_keys(bytes, area);

            let expected = vec![DiffKey {
                path: "my file.txt".to_string(),
                area,
            }];
            assert_eq!(result, expected);
        }
    }

    #[test]
    fn parse_diff_keys_ignores_empty_lines() {
        let parser = create_parser();
        let bytes = b"src/main.rs\n\n\nsrc/lib.rs\n";
        for area in [DiffArea::Untracked, DiffArea::Unstaged, DiffArea::Staged] {
            let result = parser.parse_diff_keys(bytes, area);

            let expected = vec![
                DiffKey {
                    path: "src/main.rs".to_string(),
                    area,
                },
                DiffKey {
                    path: "src/lib.rs".to_string(),
                    area,
                },
            ];
            assert_eq!(result, expected);
        }
    }

    #[test]
    fn parse_diff_keys_unicode_paths() {
        let parser = create_parser();
        let bytes = "src/日本語.rs\nsrc/é.txt\n".as_bytes();
        for area in [DiffArea::Untracked, DiffArea::Unstaged, DiffArea::Staged] {
            let result = parser.parse_diff_keys(bytes, area);

            let expected = vec![
                DiffKey {
                    path: "src/日本語.rs".to_string(),
                    area,
                },
                DiffKey {
                    path: "src/é.txt".to_string(),
                    area,
                },
            ];
            assert_eq!(result, expected);
        }
    }

    #[test]
    fn parse_hunks_strips_header_unix() {
        let parser = create_parser();
        let bytes = b"diff --git a/file.txt b/file.txt\n\
                       index 12345678..abcdef01 100644\n\
                       --- a/file.txt\n\
                       +++ b/file.txt\n\
                       @@ -1 +1 @@\n\
                       -old\n\
                       +new\n";

        let result = parser.parse_hunks(bytes);

        let expected = vec![Hunk {
            header: "@@ -1 +1 @@".to_owned(),
            lines: vec!["-old".to_owned(), "+new".to_owned()],
        }];
        assert_eq!(result, expected);
    }

    #[test]
    fn parse_hunks_strips_header_windows() {
        let parser = create_parser();
        let bytes = b"diff --git a/file.txt b/file.txt\r\n\
                       index 12345678..abcdef01 100644\r\n\
                       --- a/file.txt\r\n\
                       +++ b/file.txt\r\n\
                       @@ -1 +1 @@\r\n\
                       -old\r\n\
                       +new\r\n";

        let result = parser.parse_hunks(bytes);

        let expected = vec![Hunk {
            header: "@@ -1 +1 @@".to_owned(),
            lines: vec!["-old".to_owned(), "+new".to_owned()],
        }];
        assert_eq!(result, expected);
    }

    #[test]
    fn parse_hunks_fewer_than_four_lines() {
        let parser = create_parser();
        let bytes = b"only one line\n";

        let result = parser.parse_hunks(bytes);

        assert_eq!(result, vec![]);
    }

    #[test]
    fn parse_hunks_exactly_four_lines() {
        let parser = create_parser();
        let bytes = b"line1\nline2\nline3\nline4\n";

        let result = parser.parse_hunks(bytes);

        assert_eq!(result, vec![]);
    }

    #[test]
    fn parse_hunks_empty() {
        let parser = create_parser();
        let bytes = b"";

        let result = parser.parse_hunks(bytes);

        assert_eq!(result, vec![]);
    }
}
