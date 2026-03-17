use crate::{
    common::{self, Branch, BranchArea, DiffArea, DiffKey, DiffNumstat, Log},
    execute::Execute,
};
use anyhow::anyhow;
use std::{
    collections::HashSet,
    fs,
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

    pub fn branch(&self) -> anyhow::Result<HashSet<String>> {
        let Output { stdout, .. } = self.executor.execute_here(["branch"])?;
        Ok(common::split_by_newline(&stdout))
    }

    pub fn branch_remotes(&self) -> anyhow::Result<HashSet<String>> {
        let Output { stdout, .. } = self.executor.execute_here(["branch", "--remotes"])?;
        Ok(common::split_by_newline(&stdout))
    }

    pub fn commit(&self, message: &str) {
        self.executor
            .execute_and_log_here(["commit", "-m", message]);
    }

    pub fn diff(&self, DiffKey { path, area }: &DiffKey) -> anyhow::Result<String> {
        let Output { stdout, .. } = match area {
            DiffArea::Untracked => return Ok(fs::read_to_string(path)?),
            DiffArea::Unstaged => self.executor.execute_here(["diff", path]),
            DiffArea::Staged => self.executor.execute_here(["diff", "--staged", path]),
        }?;
        let header_end = stdout
            .iter()
            .enumerate()
            .filter(|(_, byte)| **byte == b'\n')
            .map(|(i, _)| i)
            .nth(3)
            .map_or(0, |i| i + 1);

        Ok(String::from_utf8_lossy(&stdout[header_end..]).to_string())
    }

    pub fn diff_name_only(&self) -> anyhow::Result<Vec<DiffKey>> {
        let create_keys = |stdout, area| -> Vec<DiffKey> {
            common::split_by_newline::<Vec<String>>(stdout)
                .into_iter()
                .map(|path| DiffKey { path, area })
                .collect()
        };

        let untracked_stdout = self
            .executor
            .execute_here(["ls-files", "--others", "--exclude-standard"])?
            .stdout;
        let unstaged_stdout = self.executor.execute_here(["diff", "--name-only"])?.stdout;
        let staged_stdout = self
            .executor
            .execute_here(["diff", "--staged", "--name-only"])?
            .stdout;

        let mut keys = create_keys(&untracked_stdout, DiffArea::Untracked);
        keys.extend(create_keys(&unstaged_stdout, DiffArea::Unstaged));
        keys.extend(create_keys(&staged_stdout, DiffArea::Staged));

        Ok(keys)
    }

    pub fn diff_numstat(&self, DiffKey { path, area }: &DiffKey) -> anyhow::Result<DiffNumstat> {
        let Output { stdout, .. } = match area {
            DiffArea::Untracked => {
                let lines = fs::read_to_string(path)?.lines().count();

                return Ok(DiffNumstat {
                    additions: lines.to_string(),
                    deletions: "0".to_owned(),
                });
            }
            DiffArea::Unstaged => self.executor.execute_here(["diff", "--numstat", path])?,
            DiffArea::Staged => {
                self.executor
                    .execute_here(["diff", "--staged", "--numstat", path])?
            }
        };
        let numstat: [String; 2] = common::split_whitespace_take::<Vec<String>>(&stdout, 2)
            .try_into()
            .map_err(|_| anyhow!("diff_numstat split failed"))?;
        let [additions, deletions] = numstat;

        Ok(DiffNumstat {
            additions,
            deletions,
        })
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

    pub fn pull(&self) {
        self.executor.execute_and_log_here(["pull"]);
    }

    pub fn push(&self) {
        self.executor.execute_and_log_here(["push"]);
    }

    pub fn remote(&self) -> anyhow::Result<HashSet<String>> {
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

    pub fn switch(&self, branch: &Branch) {
        let Branch { name, area } = branch;

        match area {
            BranchArea::Local => self.executor.execute_and_log_here(["switch", name]),
            BranchArea::Remote(remote) => {
                let start_point = format!("{remote}/{name}");
                self.executor
                    .execute_and_log_here(["switch", "--create", name, &start_point])
            }
        }
    }

    pub fn switch_create(&self, name: &str) {
        self.executor
            .execute_and_log_here(["switch", "--create", name]);
    }
}
