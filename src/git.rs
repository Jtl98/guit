use crate::{
    common::{self, Branch, BranchArea, DiffArea, DiffKey, DiffNumstat, Log},
    execute::Execute,
};
use std::{
    collections::BTreeSet,
    path::{Path, PathBuf},
    process::Output,
};

#[derive(Default)]
pub struct Git<E>
where
    E: Execute,
{
    executor: E,
}

impl<E> Git<E>
where
    E: Execute,
{
    pub const LOG_MAX_COUNT: usize = 100;
    const NULL: u8 = b'\x00';
    const US: u8 = b'\x1f';
    const LOG_FORMAT: &str = concat!(
        "--format=%an",
        '\x1f',
        "%aI",
        '\x1f',
        "%as",
        '\x1f',
        "%H",
        '\x1f',
        "%h",
        '\x1f',
        "%s",
        '\x1f',
        "%b",
        "%x00"
    );

    pub fn add(&self, path: &str) {
        let _ = self.executor.execute_and_log_here(["add", path]);
    }

    pub fn add_all(&self) {
        let _ = self.executor.execute_and_log_here(["add", "--all"]);
    }

    pub fn branch(&self) -> anyhow::Result<(String, BTreeSet<Branch>)> {
        let Output { stdout, .. } = self.executor.execute_here(["branch"])?;
        Ok(self.parse_local_branches(&stdout))
    }

    pub fn branch_remotes(&self) -> anyhow::Result<Vec<String>> {
        let Output { stdout, .. } = self.executor.execute_here(["branch", "--remotes"])?;
        Ok(common::split_by_newline(&stdout))
    }

    pub fn commit(&self, subject: &str) {
        let _ = self
            .executor
            .execute_and_log_here(["commit", "-m", subject]);
    }

    pub fn commit_body(&self, subject: &str, body: &str) {
        let _ = self
            .executor
            .execute_and_log_here(["commit", "-m", subject, "-m", body]);
    }

    pub fn diff(&self, path: &str) -> anyhow::Result<String> {
        let Output { stdout, .. } = self.executor.execute_here(["diff", path])?;
        Ok(self.parse_diff(&stdout))
    }

    pub fn diff_name_only(&self) -> anyhow::Result<Vec<DiffKey>> {
        let Output { stdout, .. } = self.executor.execute_here(["diff", "--name-only"])?;
        Ok(self.parse_diff_keys(&stdout, DiffArea::Unstaged))
    }

    pub fn diff_name_only_staged(&self) -> anyhow::Result<Vec<DiffKey>> {
        let Output { stdout, .. } =
            self.executor
                .execute_here(["diff", "--name-only", "--staged"])?;
        Ok(self.parse_diff_keys(&stdout, DiffArea::Staged))
    }

    pub fn diff_numstat(&self, path: &str) -> anyhow::Result<DiffNumstat> {
        let Output { stdout, .. } = self.executor.execute_here(["diff", "--numstat", path])?;
        self.parse_numstat(&stdout)
    }

    pub fn diff_numstat_staged(&self, path: &str) -> anyhow::Result<DiffNumstat> {
        let Output { stdout, .. } =
            self.executor
                .execute_here(["diff", "--numstat", "--staged", path])?;
        self.parse_numstat(&stdout)
    }

    pub fn diff_staged(&self, path: &str) -> anyhow::Result<String> {
        let Output { stdout, .. } = self.executor.execute_here(["diff", "--staged", path])?;
        Ok(self.parse_diff(&stdout))
    }

    pub fn fetch_all(&self) {
        let _ = self.executor.execute_and_log_here(["fetch", "--all"]);
    }

    pub fn init(&self, dir: &Path) {
        let _ = self
            .executor
            .execute_and_log_in(["init", "-b", "main"], dir);
    }

    pub fn log_max_count_skip(&self, skip: usize) -> anyhow::Result<Vec<Log>> {
        let Output { stdout, .. } = self.executor.execute_here([
            "log",
            "--max-count",
            &Self::LOG_MAX_COUNT.to_string(),
            "--skip",
            &skip.to_string(),
            Self::LOG_FORMAT,
        ])?;

        Ok(self.parse_logs(&stdout))
    }

    pub fn ls_files_others_exclude_standard(&self) -> anyhow::Result<Vec<DiffKey>> {
        let Output { stdout, .. } =
            self.executor
                .execute_here(["ls-files", "--others", "--exclude-standard"])?;
        Ok(self.parse_diff_keys(&stdout, DiffArea::Untracked))
    }

    pub fn pull(&self) {
        let _ = self.executor.execute_and_log_here(["pull"]);
    }

    pub fn push(&self) {
        let _ = self.executor.execute_and_log_here(["push"]);
    }

    pub fn remote(&self) -> anyhow::Result<Vec<String>> {
        let Output { stdout, .. } = self.executor.execute_here(["remote"])?;
        Ok(common::split_by_newline(&stdout))
    }

    pub fn reset_soft_head_1(&self) {
        let _ = self
            .executor
            .execute_and_log_here(["reset", "--soft", "HEAD~1"]);
    }

    pub fn restore_staged(&self, path: &str) {
        let _ = self
            .executor
            .execute_and_log_here(["restore", "--staged", path]);
    }

    pub fn restore_staged_all(&self) {
        let _ = self
            .executor
            .execute_and_log_here(["restore", "--staged", "."]);
    }

    pub fn rev_parse_show_toplevel(&self, dir: &Path) -> anyhow::Result<PathBuf> {
        let Output { stdout, .. } = self
            .executor
            .execute_in(["rev-parse", "--show-toplevel"], dir)?;
        let trimmed = stdout.trim_ascii_end();
        let lossy = String::from_utf8_lossy(trimmed).to_string();

        Ok(PathBuf::from(lossy))
    }

    pub fn stash_pop_index(&self) {
        let _ = self
            .executor
            .execute_and_log_here(["stash", "pop", "--index"]);
    }

    pub fn stash_push_include_untracked(&self) {
        let _ = self
            .executor
            .execute_and_log_here(["stash", "push", "--include-untracked"]);
    }

    pub fn switch(&self, branch: &str) {
        let _ = self.executor.execute_and_log_here(["switch", branch]);
    }

    pub fn switch_create(&self, branch: &str) {
        let _ = self
            .executor
            .execute_and_log_here(["switch", "--create", branch]);
    }

    pub fn switch_create_remote(&self, branch: &str, remote: &str) {
        let start_point = format!("{remote}/{branch}");
        let _ = self
            .executor
            .execute_and_log_here(["switch", "--create", branch, &start_point]);
    }

    fn parse_diff(&self, stdout: &[u8]) -> String {
        String::from_utf8_lossy(stdout)
            .lines()
            .skip(4)
            .collect::<Vec<&str>>()
            .join("\n")
    }

    fn parse_diff_keys(&self, stdout: &[u8], area: DiffArea) -> Vec<DiffKey> {
        common::split_by_newline::<Vec<String>>(stdout)
            .into_iter()
            .map(|path| DiffKey { path, area })
            .collect()
    }

    fn parse_local_branches(&self, stdout: &[u8]) -> (String, BTreeSet<Branch>) {
        let mut current = String::new();
        let mut other = BTreeSet::new();

        let branches = common::split_by_newline::<Vec<String>>(stdout);
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

    fn parse_logs(&self, stdout: &[u8]) -> Vec<Log> {
        common::split_by_byte(stdout, Self::NULL)
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

    fn parse_numstat(&self, stdout: &[u8]) -> anyhow::Result<DiffNumstat> {
        let [additions, deletions] = common::split_whitespace_take::<2>(stdout)?;

        Ok(DiffNumstat {
            additions,
            deletions,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::execute::tests::MockExecutor;

    fn create_git() -> Git<MockExecutor> {
        Git::<MockExecutor>::default()
    }

    #[test]
    fn parse_diff_strips_header_unix() {
        let git = create_git();
        let stdout = b"diff --git a/file.txt b/file.txt\n\
                       index 12345678..abcdef01 100644\n\
                       --- a/file.txt\n\
                       +++ b/file.txt\n\
                       @@ -1 +1 @@\n\
                       -old\n\
                       +new\n";
        let result = git.parse_diff(stdout);

        assert_eq!(result, "@@ -1 +1 @@\n-old\n+new");
    }

    #[test]
    fn parse_diff_strips_header_windows() {
        let git = create_git();
        let stdout = b"diff --git a/file.txt b/file.txt\r\n\
                       index 12345678..abcdef01 100644\r\n\
                       --- a/file.txt\r\n\
                       +++ b/file.txt\r\n\
                       @@ -1 +1 @@\r\n\
                       -old\r\n\
                       +new\r\n";
        let result = git.parse_diff(stdout);

        assert_eq!(result, "@@ -1 +1 @@\n-old\n+new");
    }

    #[test]
    fn parse_diff_fewer_than_four_lines() {
        let git = create_git();
        let stdout = b"only one line\n";
        let result = git.parse_diff(stdout);

        assert_eq!(result, "");
    }

    #[test]
    fn parse_diff_exactly_four_lines() {
        let git = create_git();
        let stdout = b"line1\nline2\nline3\nline4\n";
        let result = git.parse_diff(stdout);

        assert_eq!(result, "");
    }

    #[test]
    fn parse_diff_empty() {
        let git = create_git();
        let stdout = b"";
        let result = git.parse_diff(stdout);

        assert_eq!(result, "");
    }

    #[test]
    fn parse_diff_keys_empty_stdout() {
        let git = create_git();
        let stdout = b"";
        for area in [DiffArea::Untracked, DiffArea::Unstaged, DiffArea::Staged] {
            let keys = git.parse_diff_keys(stdout, area);

            assert!(keys.is_empty());
        }
    }

    #[test]
    fn parse_diff_keys_single_path() {
        let git = create_git();
        let stdout = b"src/main.rs\n";
        for area in [DiffArea::Untracked, DiffArea::Unstaged, DiffArea::Staged] {
            let keys = git.parse_diff_keys(stdout, area);

            let expected = vec![DiffKey {
                path: "src/main.rs".to_string(),
                area,
            }];
            assert_eq!(keys, expected);
        }
    }

    #[test]
    fn parse_diff_keys_multiple_paths() {
        let git = create_git();
        let stdout = b"src/main.rs\nsrc/lib.rs\nCargo.toml\n";
        for area in [DiffArea::Untracked, DiffArea::Unstaged, DiffArea::Staged] {
            let keys = git.parse_diff_keys(stdout, area);

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
            assert_eq!(keys, expected);
        }
    }

    #[test]
    fn parse_diff_keys_path_with_spaces() {
        let git = create_git();
        let stdout = b"my file.txt\n";
        for area in [DiffArea::Untracked, DiffArea::Unstaged, DiffArea::Staged] {
            let keys = git.parse_diff_keys(stdout, area);

            let expected = vec![DiffKey {
                path: "my file.txt".to_string(),
                area,
            }];
            assert_eq!(keys, expected);
        }
    }

    #[test]
    fn parse_diff_keys_ignores_empty_lines() {
        let git = create_git();
        let stdout = b"src/main.rs\n\n\nsrc/lib.rs\n";
        for area in [DiffArea::Untracked, DiffArea::Unstaged, DiffArea::Staged] {
            let keys = git.parse_diff_keys(stdout, area);

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
            assert_eq!(keys, expected);
        }
    }

    #[test]
    fn parse_diff_keys_unicode_paths() {
        let git = create_git();
        let stdout = "src/日本語.rs\nsrc/é.txt\n".as_bytes();
        for area in [DiffArea::Untracked, DiffArea::Unstaged, DiffArea::Staged] {
            let keys = git.parse_diff_keys(stdout, area);

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
            assert_eq!(keys, expected);
        }
    }
}
