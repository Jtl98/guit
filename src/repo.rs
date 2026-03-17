use crate::{
    common::{Branch, BranchArea, Branches, DatedLogs, Diff, DiffArea, Diffs, Log},
    execute::GitExecutor,
    git::Git,
};
use std::{cmp::Reverse, collections::BTreeSet, fs, path::PathBuf, process::Output};

#[derive(Default)]
pub struct Repo {
    pub dir: PathBuf,
    pub diffs: Diffs,
    pub branches: Branches,
    pub dated_logs: DatedLogs,
    logs_skipped: usize,
}

impl Repo {
    pub fn new(git: &Git<GitExecutor>, dir: PathBuf) -> anyhow::Result<Self> {
        let diffs = Self::diffs(git)?;
        let branches = Self::branches(git)?;
        let logs_skipped = 0;
        let dated_logs =
            git.log(logs_skipped)?
                .into_iter()
                .fold(DatedLogs::new(), |mut logs, log| {
                    Self::add_log(&mut logs, log);
                    logs
                });

        Ok(Self {
            dir,
            diffs,
            branches,
            dated_logs,
            logs_skipped,
        })
    }

    pub fn load_logs(&mut self, git: &Git<GitExecutor>) -> anyhow::Result<()> {
        let skip = self.logs_skipped + Git::<GitExecutor>::LOG_MAX_COUNT;
        let logs = git.log(skip)?;
        self.logs_skipped += logs.len();

        for log in logs {
            Self::add_log(&mut self.dated_logs, log);
        }

        Ok(())
    }

    fn add_log(logs: &mut DatedLogs, log: Log) {
        let date = Reverse(log.short_date.clone());
        logs.entry(date).or_default().push(log);
    }

    fn branches(git: &Git<GitExecutor>) -> anyhow::Result<Branches> {
        let local_branches = git.branch()?;
        let remote_branches = git.branch_remotes()?;
        let remotes = git.remote()?;

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

    fn diffs(git: &Git<GitExecutor>) -> anyhow::Result<Diffs> {
        let mut diffs = Diffs::new();

        for key in git.diff_name_only()? {
            let content = match key.area {
                DiffArea::Untracked => fs::read_to_string(&key.path)?,
                DiffArea::Unstaged => {
                    let Output { stdout, .. } = git.diff(&key.path)?;
                    Self::strip_diff_header(&stdout)
                }
                DiffArea::Staged => {
                    let Output { stdout, .. } = git.diff_staged(&key.path)?;
                    Self::strip_diff_header(&stdout)
                }
            };

            let numstat = git.diff_numstat(&key)?;
            let diff = Diff { content, numstat };

            diffs.insert(key, diff);
        }

        Ok(diffs)
    }

    fn strip_diff_header(stdout: &[u8]) -> String {
        let header_end = stdout
            .iter()
            .enumerate()
            .filter(|(_, byte)| **byte == b'\n')
            .map(|(i, _)| i)
            .nth(3)
            .map_or(0, |i| i + 1);
        let diff = &stdout[header_end..];

        String::from_utf8_lossy(diff).to_string()
    }
}
