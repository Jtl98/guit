use crate::{
    common::{Branches, DiffKey, Logs},
    git::Git,
};
use std::{cmp::Reverse, collections::BTreeMap, path::PathBuf};

type Diffs = BTreeMap<DiffKey, String>;

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
            logs.entry(date).or_insert(vec![]).push(log);
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
