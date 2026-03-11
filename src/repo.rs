use crate::{
    common::{Branches, Diffs, Logs},
    git::Git,
};
use std::{cmp::Reverse, path::PathBuf};

#[derive(Default)]
pub struct Repo {
    pub dir: PathBuf,
    pub diffs: Diffs,
    pub branches: Branches,
    pub logs: Logs,
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
        let logs = git.log()?.into_iter().fold(Logs::new(), |mut logs, log| {
            let date = Reverse(log.short_date.clone());
            logs.entry(date).or_insert_with(Vec::new).push(log);
            logs
        });

        Ok(Self {
            dir,
            diffs,
            branches,
            logs,
        })
    }
}
