use crate::common::{self, Branch, BranchArea, Branches, DiffArea, DiffKey, DiffNumstat, Log};
use anyhow::anyhow;
use log::{error, info};
use std::{
    collections::{BTreeSet, HashSet},
    ffi::OsStr,
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
};

#[derive(Default)]
pub struct Git;

impl Git {
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

    pub fn add_or_restore(&self, key: &DiffKey) {
        match key.area {
            DiffArea::Untracked | DiffArea::Unstaged => {
                self.execute_and_log_here(["add", &key.path])
            }
            DiffArea::Staged => self.execute_and_log_here(["restore", "--staged", &key.path]),
        }
    }

    pub fn branches(&self) -> anyhow::Result<Branches> {
        let local_branches = self.branch()?;
        let remote_branches = self.branch_remotes()?;
        let remotes = self.remote()?;

        let mut current = String::new();
        let mut other = BTreeSet::new();

        for branch in &local_branches {
            let trimmed = branch[2..].to_owned();

            if branch.starts_with("* ") {
                current = trimmed;
            } else {
                other.insert(Branch {
                    name: trimmed,
                    area: BranchArea::Local,
                });
            }
        }

        for branch in &remote_branches {
            for remote in &remotes {
                if let Some(name) = branch.strip_prefix(&format!("  {remote}/"))
                    && !name.contains(' ')
                    && name != current
                {
                    other.insert(Branch {
                        name: name.to_owned(),
                        area: BranchArea::Remote(remote.to_owned()),
                    });
                }
            }
        }

        Ok(Branches { current, other })
    }

    pub fn commit(&self, message: &str) {
        self.execute_and_log_here(["commit", "-m", message]);
    }

    pub fn diff(&self, DiffKey { path, area }: &DiffKey) -> anyhow::Result<String> {
        let Output { stdout, .. } = match area {
            DiffArea::Untracked => return Ok(fs::read_to_string(path)?),
            DiffArea::Unstaged => self.execute_here(["diff", path]),
            DiffArea::Staged => self.execute_here(["diff", "--staged", path]),
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
            .execute_here(["ls-files", "--others", "--exclude-standard"])?
            .stdout;
        let unstaged_stdout = self.execute_here(["diff", "--name-only"])?.stdout;
        let staged_stdout = self
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
            DiffArea::Unstaged => self.execute_here(["diff", "--numstat", path])?,
            DiffArea::Staged => self.execute_here(["diff", "--staged", "--numstat", path])?,
        };
        let numstat: [String; 3] = common::split_by_whitespace::<Vec<String>>(&stdout)
            .try_into()
            .map_err(|_| anyhow!("diff_numstat parsing failed"))?;
        let [additions, deletions, _path] = numstat;

        Ok(DiffNumstat {
            additions,
            deletions,
        })
    }

    pub fn fetch_all(&self) {
        self.execute_and_log_here(["fetch", "--all"]);
    }

    pub fn init(&self, dir: &Path) {
        self.execute_and_log_in(["init", "-b", "main"], dir);
    }

    pub fn log(&self, skip: usize) -> anyhow::Result<Vec<Log>> {
        let Output { stdout, .. } = self.execute_here([
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
        self.execute_and_log_here(["pull"]);
    }

    pub fn push(&self) {
        self.execute_and_log_here(["push"]);
    }

    pub fn reset_soft_head_1(&self) {
        self.execute_and_log_here(["reset", "--soft", "HEAD~1"]);
    }

    pub fn rev_parse_show_toplevel(&self, dir: &Path) -> anyhow::Result<PathBuf> {
        let Output { stdout, .. } = self.execute_in(["rev-parse", "--show-toplevel"], dir)?;
        let trimmed = stdout.trim_ascii_end();
        let lossy = String::from_utf8_lossy(trimmed).to_string();

        Ok(PathBuf::from(lossy))
    }

    pub fn switch(&self, branch: &Branch) {
        let Branch { name, area } = branch;

        match area {
            BranchArea::Local => self.execute_and_log_here(["switch", name]),
            BranchArea::Remote(remote) => {
                let start_point = format!("{remote}/{name}");
                self.execute_and_log_here(["switch", "--create", name, &start_point])
            }
        }
    }

    pub fn switch_create(&self, name: &str) {
        self.execute_and_log_here(["switch", "--create", name]);
    }

    fn branch(&self) -> anyhow::Result<HashSet<String>> {
        let Output { stdout, .. } = self.execute_here(["branch"])?;
        Ok(common::split_by_newline(&stdout))
    }

    fn branch_remotes(&self) -> anyhow::Result<HashSet<String>> {
        let Output { stdout, .. } = self.execute_here(["branch", "--remotes"])?;
        Ok(common::split_by_newline(&stdout))
    }

    fn remote(&self) -> anyhow::Result<HashSet<String>> {
        let Output { stdout, .. } = self.execute_here(["remote"])?;
        Ok(common::split_by_newline(&stdout))
    }

    fn execute<I, S>(&self, args: I, dir: Option<&Path>) -> anyhow::Result<Output>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        let mut command = Command::new("git");
        command.args(args);

        if let Some(dir) = dir {
            command.current_dir(dir);
        }

        Ok(command.output()?)
    }

    fn execute_in<I, S>(&self, args: I, dir: &Path) -> anyhow::Result<Output>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        self.execute(args, Some(dir))
    }

    fn execute_here<I, S>(&self, args: I) -> anyhow::Result<Output>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        self.execute(args, None)
    }

    fn execute_and_log<I, S>(&self, args: I, dir: Option<&Path>)
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        match self.execute(args, dir) {
            Ok(Output {
                status,
                stdout,
                stderr,
            }) => {
                let stdout = String::from_utf8_lossy(&stdout);
                let stderr = String::from_utf8_lossy(&stderr);

                if status.success() {
                    info!("{}", stdout);
                    info!("{}", stderr);
                } else {
                    error!("{}", stdout);
                    error!("{}", stderr);
                }
            }
            Err(error) => error!("{}", error),
        }
    }

    fn execute_and_log_in<I, S>(&self, args: I, dir: &Path)
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        self.execute_and_log(args, Some(dir));
    }

    fn execute_and_log_here<I, S>(&self, args: I)
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        self.execute_and_log(args, None);
    }
}
