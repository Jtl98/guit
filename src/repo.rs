use crate::{
    common::{Branches, DiffKey, Log},
    git::Git,
};
use std::{
    cmp::Reverse,
    collections::{BTreeMap, HashMap},
    path::PathBuf,
};

type Diffs = HashMap<DiffKey, String>;
type Logs = BTreeMap<Reverse<String>, Vec<Log>>;

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
        let logs = git
            .log()?
            .into_iter()
            .fold(BTreeMap::new(), |mut map, log| {
                let date = log.short_date.clone();
                let logs = map.entry(Reverse(date)).or_insert(Vec::new());
                logs.push(log);
                map
            });

        Ok(Self {
            dir,
            diffs,
            branches,
            logs,
        })
    }
}
