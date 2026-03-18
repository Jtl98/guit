use crate::{
    common::{self, DiffArea, DiffKey, DiffNumstat, Log},
    execute::Execute,
};
use anyhow::anyhow;
use std::{
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
        "%s"
    );

    pub fn add(&self, path: &str) {
        self.executor.execute_and_log_here(["add", path]);
    }

    pub fn branch(&self) -> anyhow::Result<Vec<String>> {
        let Output { stdout, .. } = self.executor.execute_here(["branch"])?;
        Ok(common::split_by_newline(&stdout))
    }

    pub fn branch_remotes(&self) -> anyhow::Result<Vec<String>> {
        let Output { stdout, .. } = self.executor.execute_here(["branch", "--remotes"])?;
        Ok(common::split_by_newline(&stdout))
    }

    pub fn commit(&self, message: &str) {
        self.executor
            .execute_and_log_here(["commit", "-m", message]);
    }

    pub fn diff(&self, path: &str) -> anyhow::Result<Output> {
        self.executor.execute_here(["diff", path])
    }

    pub fn diff_name_only(&self) -> anyhow::Result<Vec<DiffKey>> {
        let Output { stdout, .. } = self.executor.execute_here(["diff", "--name-only"])?;
        Ok(self.create_diff_keys(&stdout, DiffArea::Unstaged))
    }

    pub fn diff_name_only_staged(&self) -> anyhow::Result<Vec<DiffKey>> {
        let Output { stdout, .. } =
            self.executor
                .execute_here(["diff", "--name-only", "--staged"])?;
        Ok(self.create_diff_keys(&stdout, DiffArea::Staged))
    }

    pub fn diff_numstat(&self, path: &str) -> anyhow::Result<DiffNumstat> {
        let Output { stdout, .. } = self.executor.execute_here(["diff", "--numstat", path])?;
        self.create_numstat(&stdout)
    }

    pub fn diff_numstat_staged(&self, path: &str) -> anyhow::Result<DiffNumstat> {
        let Output { stdout, .. } =
            self.executor
                .execute_here(["diff", "--numstat", "--staged", path])?;
        self.create_numstat(&stdout)
    }

    pub fn diff_staged(&self, path: &str) -> anyhow::Result<Output> {
        self.executor.execute_here(["diff", "--staged", path])
    }

    pub fn fetch_all(&self) {
        self.executor.execute_and_log_here(["fetch", "--all"]);
    }

    pub fn init(&self, dir: &Path) {
        self.executor
            .execute_and_log_in(["init", "-b", "main"], dir);
    }

    pub fn log(&self, skip: usize) -> anyhow::Result<Vec<Log>> {
        let Output { stdout, .. } = self.executor.execute_here([
            "log",
            "--max-count",
            &Self::LOG_MAX_COUNT.to_string(),
            "--skip",
            &skip.to_string(),
            Self::LOG_FORMAT,
        ])?;
        let logs = common::split_by_newline::<Vec<String>>(&stdout)
            .into_iter()
            .filter_map(|log| {
                let mut parts = log.split('\x1f');
                let author = parts.next()?.to_owned();
                let long_date = parts.next()?.to_owned();
                let short_date = parts.next()?.to_owned();
                let long_hash = parts.next()?.to_owned();
                let short_hash = parts.next()?.to_owned();
                let subject = parts.next()?.to_owned();

                Some(Log {
                    author,
                    long_date,
                    short_date,
                    long_hash,
                    short_hash,
                    subject,
                })
            })
            .collect();

        Ok(logs)
    }

    pub fn ls_files_others_exclude_standard(&self) -> anyhow::Result<Vec<DiffKey>> {
        let Output { stdout, .. } =
            self.executor
                .execute_here(["ls-files", "--others", "--exclude-standard"])?;
        Ok(self.create_diff_keys(&stdout, DiffArea::Untracked))
    }

    pub fn pull(&self) {
        self.executor.execute_and_log_here(["pull"]);
    }

    pub fn push(&self) {
        self.executor.execute_and_log_here(["push"]);
    }

    pub fn remote(&self) -> anyhow::Result<Vec<String>> {
        let Output { stdout, .. } = self.executor.execute_here(["remote"])?;
        Ok(common::split_by_newline(&stdout))
    }

    pub fn reset_soft_head_1(&self) {
        self.executor
            .execute_and_log_here(["reset", "--soft", "HEAD~1"]);
    }

    pub fn restore_staged(&self, path: &str) {
        self.executor
            .execute_and_log_here(["restore", "--staged", path]);
    }

    pub fn rev_parse_show_toplevel(&self, dir: &Path) -> anyhow::Result<PathBuf> {
        let Output { stdout, .. } = self
            .executor
            .execute_in(["rev-parse", "--show-toplevel"], dir)?;
        let trimmed = stdout.trim_ascii_end();
        let lossy = String::from_utf8_lossy(trimmed).to_string();

        Ok(PathBuf::from(lossy))
    }

    pub fn stash_push_include_untracked(&self) {
        self.executor
            .execute_and_log_here(["stash", "push", "--include-untracked"]);
    }

    pub fn switch(&self, branch: &str) {
        self.executor.execute_and_log_here(["switch", branch]);
    }

    pub fn switch_create(&self, branch: &str) {
        self.executor
            .execute_and_log_here(["switch", "--create", branch]);
    }

    pub fn switch_create_remote(&self, branch: &str, remote: &str) {
        let start_point = format!("{remote}/{branch}");
        self.executor
            .execute_and_log_here(["switch", "--create", branch, &start_point]);
    }

    fn create_diff_keys(&self, stdout: &[u8], area: DiffArea) -> Vec<DiffKey> {
        common::split_by_newline::<Vec<String>>(stdout)
            .into_iter()
            .map(|path| DiffKey { path, area })
            .collect()
    }

    fn create_numstat(&self, stdout: &[u8]) -> anyhow::Result<DiffNumstat> {
        let numstat: [String; 2] = common::split_whitespace_take::<Vec<String>>(stdout, 2)
            .try_into()
            .map_err(|_| anyhow!("create_numstat split failed"))?;
        let [additions, deletions] = numstat;

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
    fn create_diff_keys_empty_stdout() {
        let git = create_git();
        let stdout = b"";
        for area in [DiffArea::Untracked, DiffArea::Unstaged, DiffArea::Staged] {
            let keys = git.create_diff_keys(stdout, area);

            assert!(keys.is_empty());
        }
    }

    #[test]
    fn create_diff_keys_single_path() {
        let git = create_git();
        let stdout = b"src/main.rs\n";
        for area in [DiffArea::Untracked, DiffArea::Unstaged, DiffArea::Staged] {
            let keys = git.create_diff_keys(stdout, area);

            let expected = vec![DiffKey {
                path: "src/main.rs".to_string(),
                area,
            }];
            assert_eq!(keys, expected);
        }
    }

    #[test]
    fn create_diff_keys_multiple_paths() {
        let git = create_git();
        let stdout = b"src/main.rs\nsrc/lib.rs\nCargo.toml\n";
        for area in [DiffArea::Untracked, DiffArea::Unstaged, DiffArea::Staged] {
            let keys = git.create_diff_keys(stdout, area);

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
    fn create_diff_keys_path_with_spaces() {
        let git = create_git();
        let stdout = b"my file.txt\n";
        for area in [DiffArea::Untracked, DiffArea::Unstaged, DiffArea::Staged] {
            let keys = git.create_diff_keys(stdout, area);

            let expected = vec![DiffKey {
                path: "my file.txt".to_string(),
                area,
            }];
            assert_eq!(keys, expected);
        }
    }

    #[test]
    fn create_diff_keys_ignores_empty_lines() {
        let git = create_git();
        let stdout = b"src/main.rs\n\n\nsrc/lib.rs\n";
        for area in [DiffArea::Untracked, DiffArea::Unstaged, DiffArea::Staged] {
            let keys = git.create_diff_keys(stdout, area);

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
    fn create_diff_keys_unicode_paths() {
        let git = create_git();
        let stdout = "src/日本語.rs\nsrc/é.txt\n".as_bytes();
        for area in [DiffArea::Untracked, DiffArea::Unstaged, DiffArea::Staged] {
            let keys = git.create_diff_keys(stdout, area);

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
