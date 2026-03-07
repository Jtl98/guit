use crate::{
    common::{Branches, DiffKey, Log},
    git::Git,
};
use std::{collections::HashMap, path::PathBuf};

type Diffs = HashMap<DiffKey, String>;

#[derive(Default)]
pub struct Repo {
    pub dir: PathBuf,
    pub diffs: Diffs,
    pub branches: Branches,
    pub logs: Vec<Log>,
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
        let logs = git.log()?;

        Ok(Self {
            dir,
            diffs,
            branches,
            logs,
        })
    }
}
