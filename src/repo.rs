use crate::{common::DiffKey, git::Git};
use std::{collections::HashMap, io};

type Diffs = HashMap<DiffKey, String>;

#[derive(Default)]
pub struct Repo {
    pub diffs: Diffs,
    pub current_branch: String,
}

impl Repo {
    pub fn new(git: &Git) -> io::Result<Self> {
        let diffs = git
            .diff_name_only()?
            .into_iter()
            .map(|key| {
                let diff = git.diff(&key)?;
                Ok((key, diff))
            })
            .collect::<io::Result<Diffs>>()?;
        let current_branch = git.current_branch()?;

        Ok(Self {
            diffs,
            current_branch,
        })
    }
}
