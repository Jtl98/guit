use crate::{
    common::{Branches, Diffs, Log, Logs},
    git::Git,
};
use std::{cmp::Reverse, path::PathBuf};

#[derive(Default)]
pub struct Repo {
    pub dir: PathBuf,
    pub diffs: Diffs,
    pub branches: Branches,
    pub logs: Logs,
    logs_skipped: usize,
}

impl Repo {
    pub fn new(git: &Git, dir: PathBuf) -> anyhow::Result<Self> {
        let diffs = git
            .diff_name_only()?
            .into_iter()
            .map(|key| {
                let diff = git.diff(&key)?;
                Ok((key, diff))
            })
            .collect::<anyhow::Result<Diffs>>()?;
        let branches = git.branches()?;
        let logs_skipped = 0;
        let logs = git
            .log(logs_skipped)?
            .into_iter()
            .fold(Logs::new(), |mut logs, log| {
                Self::add_log(&mut logs, log);
                logs
            });

        Ok(Self {
            dir,
            diffs,
            branches,
            logs,
            logs_skipped,
        })
    }

    pub fn load_logs(&mut self, git: &Git) -> anyhow::Result<()> {
        let skip = self.logs_skipped + Git::LOG_MAX_COUNT;
        let logs = git.log(skip)?;
        self.logs_skipped += logs.len();

        for log in logs {
            Self::add_log(&mut self.logs, log);
        }

        Ok(())
    }

    fn add_log(logs: &mut Logs, log: Log) {
        let date = Reverse(log.short_date.clone());
        logs.entry(date).or_default().push(log);
    }
}
