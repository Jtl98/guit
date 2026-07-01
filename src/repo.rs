use crate::{
    common::{
        Branch, BranchArea, Branches, DatedLogs, Diff, DiffArea, Diffs, HunkDiff, Log, StringDiff,
    },
    executor::GitExecutor,
    git::Git,
};
use git2::{BranchType, Repository};
use std::{cmp::Reverse, collections::BTreeSet, fs, path::PathBuf};

pub struct Repo {
    pub dir: PathBuf,
    pub diffs: Diffs,
    pub branches: Branches,
    pub dated_logs: DatedLogs,
    logs_skipped: usize,
}

impl Repo {
    const HEAD: &str = "HEAD";

    pub fn new(git: &Git<GitExecutor>, dir: PathBuf) -> anyhow::Result<Self> {
        let repository = Repository::open(&dir)?;
        let diffs = Self::diffs(git)?;
        let branches = Self::branches(&repository)?;
        let logs_skipped = 0;
        let dated_logs = git.log_max_count_skip(logs_skipped)?.into_iter().fold(
            DatedLogs::new(),
            |mut logs, log| {
                Self::add_log(&mut logs, log);
                logs
            },
        );

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
        let logs = git.log_max_count_skip(skip)?;
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

    fn branches(repository: &Repository) -> anyhow::Result<Branches> {
        let head = repository.head()?;
        let current = if repository.head_detached()? {
            match head.target() {
                Some(oid) => &format!("detached at {}", oid),
                None => "detached",
            }
        } else {
            head.shorthand()?
        };

        let is_current_or_head = |name: &str| name == current || name == Self::HEAD;
        let mut other = BTreeSet::new();

        let local_branches = repository.branches(Some(BranchType::Local))?;
        for local_branch in local_branches {
            let (branch, _) = local_branch?;
            let name = branch.get().shorthand()?;

            if is_current_or_head(name) {
                continue;
            }

            other.insert(Branch {
                name: name.to_owned(),
                area: BranchArea::Local,
            });
        }

        let remote_branches = repository.branches(Some(BranchType::Remote))?;
        for remote_branch in remote_branches {
            let (branch, _) = remote_branch?;
            let refname = branch.get().name()?;
            let remote = repository.branch_remote_name(refname)?;
            let remote = remote.as_str()?;
            let remote_delimiter = format!("{}/", remote);

            let Some((_, name)) = refname.split_once(&remote_delimiter) else {
                continue;
            };

            if is_current_or_head(name) {
                continue;
            }

            other.insert(Branch {
                name: name.to_owned(),
                area: BranchArea::Remote(remote.to_owned()),
            });
        }

        Ok(Branches {
            current: current.to_owned(),
            other,
        })
    }

    fn diffs(git: &Git<GitExecutor>) -> anyhow::Result<Diffs> {
        let mut diffs = Diffs::new();
        let untracked_keys = git.ls_files_others_exclude_standard()?;
        let unstaged_keys = git.diff_name_only()?;
        let staged_keys = git.diff_name_only_staged()?;

        let keys = untracked_keys
            .into_iter()
            .chain(unstaged_keys)
            .chain(staged_keys);

        for key in keys {
            let Ok(content) = fs::read_to_string(&key.path) else {
                diffs.insert(key, Diff::Binary);
                continue;
            };

            let diff = match key.area {
                DiffArea::Untracked => {
                    let lines = content.lines().map(str::to_owned).collect();

                    Diff::String(StringDiff { lines })
                }
                DiffArea::Unstaged => {
                    let hunks = git.diff(&key.path)?;
                    let numstat = git.diff_numstat(&key.path)?;

                    Diff::Hunk(HunkDiff { hunks, numstat })
                }
                DiffArea::Staged => {
                    let hunks = git.diff_staged(&key.path)?;
                    let numstat = git.diff_numstat_staged(&key.path)?;

                    Diff::Hunk(HunkDiff { hunks, numstat })
                }
            };

            diffs.insert(key, diff);
        }

        Ok(diffs)
    }
}
