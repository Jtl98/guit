use crate::{
    common::{Branches, DatedLogs, Diffs, Log},
    git::Git,
};
use std::{cmp::Reverse, path::PathBuf};

#[derive(Default)]
pub struct Repo {
    pub dir: PathBuf,
    pub diffs: Diffs,
    pub branches: Branches,
    pub dated_logs: DatedLogs,
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

    pub fn load_logs(&mut self, git: &Git) -> anyhow::Result<()> {
        let skip = self.logs_skipped + Git::LOG_MAX_COUNT;
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
}
