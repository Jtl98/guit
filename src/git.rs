use crate::{
    common::{self, Branch, DiffArea, DiffKey, DiffNumstat, Hunk, Log},
    executor::Execute,
    parser::GitParser,
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
    parser: GitParser,
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
        let branches = self.parser.parse_local_branches(&stdout);

        Ok(branches)
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

    pub fn diff(&self, path: &str) -> anyhow::Result<Vec<Hunk>> {
        let Output { stdout, .. } = self.executor.execute_here(["diff", path])?;
        let hunks = self.parser.parse_hunks(&stdout);

        Ok(hunks)
    }

    pub fn diff_name_only(&self) -> anyhow::Result<Vec<DiffKey>> {
        let Output { stdout, .. } = self.executor.execute_here(["diff", "--name-only"])?;
        let keys = self.parser.parse_diff_keys(&stdout, DiffArea::Unstaged);

        Ok(keys)
    }

    pub fn diff_name_only_staged(&self) -> anyhow::Result<Vec<DiffKey>> {
        let Output { stdout, .. } =
            self.executor
                .execute_here(["diff", "--name-only", "--staged"])?;
        let keys = self.parser.parse_diff_keys(&stdout, DiffArea::Staged);

        Ok(keys)
    }

    pub fn diff_numstat(&self, path: &str) -> anyhow::Result<DiffNumstat> {
        let Output { stdout, .. } = self.executor.execute_here(["diff", "--numstat", path])?;

        self.parser.parse_numstat(&stdout)
    }

    pub fn diff_numstat_staged(&self, path: &str) -> anyhow::Result<DiffNumstat> {
        let Output { stdout, .. } =
            self.executor
                .execute_here(["diff", "--numstat", "--staged", path])?;

        self.parser.parse_numstat(&stdout)
    }

    pub fn diff_staged(&self, path: &str) -> anyhow::Result<Vec<Hunk>> {
        let Output { stdout, .. } = self.executor.execute_here(["diff", "--staged", path])?;
        let hunks = self.parser.parse_hunks(&stdout);

        Ok(hunks)
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
        let logs = self.parser.parse_logs(&stdout);

        Ok(logs)
    }

    pub fn ls_files_others_exclude_standard(&self) -> anyhow::Result<Vec<DiffKey>> {
        let Output { stdout, .. } =
            self.executor
                .execute_here(["ls-files", "--others", "--exclude-standard"])?;
        let keys = self.parser.parse_diff_keys(&stdout, DiffArea::Untracked);

        Ok(keys)
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
}
