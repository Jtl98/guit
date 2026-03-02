use crate::common::{Branch, BranchArea, Branches, DiffArea, DiffKey};
use log::{error, info};
use std::{
    collections::HashSet,
    ffi::OsStr,
    fs,
    io::{self},
    path::Path,
    process::{Command, Output},
};

#[derive(Default)]
pub struct Git;

impl Git {
    pub fn add_or_restore(&self, key: &DiffKey) {
        match key.area {
            DiffArea::Untracked | DiffArea::Unstaged => self.execute_and_log(["add", &key.path]),
            DiffArea::Staged => self.execute_and_log(["restore", "--staged", &key.path]),
        }
    }

    pub fn branches(&self) -> io::Result<Branches> {
        let local_branches = self.branch()?;
        let remote_branches = self.branch_remotes()?;
        let remotes = self.remote()?;

        let mut current = String::new();
        let mut other = HashSet::new();

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
        self.execute_and_log(["commit", "-m", message]);
    }

    pub fn diff(&self, DiffKey { path, area }: &DiffKey) -> io::Result<String> {
        let Output { stdout, .. } = match area {
            DiffArea::Untracked => return fs::read_to_string(path),
            DiffArea::Unstaged => self.execute(["diff", path]),
            DiffArea::Staged => self.execute(["diff", "--staged", path]),
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

    pub fn diff_name_only(&self) -> io::Result<Vec<DiffKey>> {
        let create_keys = |stdout, area| -> Vec<DiffKey> {
            self.split_by_newline_vec(stdout)
                .into_iter()
                .map(|path| DiffKey { path, area })
                .collect()
        };

        let untracked_stdout = self
            .execute(["ls-files", "--others", "--exclude-standard"])?
            .stdout;
        let unstaged_stdout = self.execute(["diff", "--name-only"])?.stdout;
        let staged_stdout = self.execute(["diff", "--staged", "--name-only"])?.stdout;

        let mut keys = create_keys(&untracked_stdout, DiffArea::Untracked);
        keys.extend(create_keys(&unstaged_stdout, DiffArea::Unstaged));
        keys.extend(create_keys(&staged_stdout, DiffArea::Staged));

        Ok(keys)
    }

    pub fn fetch_all(&self) {
        self.execute_and_log(["fetch", "--all"])
    }

    pub fn pull(&self) {
        self.execute_and_log(["pull"])
    }

    pub fn push(&self) {
        self.execute_and_log(["push"])
    }

    pub fn rev_parse_show_toplevel<P: AsRef<Path>>(&self, dir: P) -> io::Result<String> {
        let Output { stdout, .. } = self.execute_in_dir(["rev-parse", "--show-toplevel"], dir)?;
        let trimmed = stdout.trim_ascii();

        Ok(String::from_utf8_lossy(trimmed).to_string())
    }

    pub fn switch(&self, branch: &Branch) {
        let Branch { name, area } = branch;

        match area {
            BranchArea::Local => self.execute_and_log(["switch", name]),
            BranchArea::Remote(remote) => {
                let start_point = format!("{remote}/{name}");
                self.execute_and_log(["switch", "--create", name, &start_point])
            }
        }
    }

    fn branch(&self) -> io::Result<HashSet<String>> {
        let Output { stdout, .. } = self.execute(["branch"])?;
        Ok(self.split_by_newline(&stdout))
    }

    fn branch_remotes(&self) -> io::Result<HashSet<String>> {
        let Output { stdout, .. } = self.execute(["branch", "--remotes"])?;
        Ok(self.split_by_newline(&stdout))
    }

    fn remote(&self) -> io::Result<HashSet<String>> {
        let Output { stdout, .. } = self.execute(["remote"])?;
        Ok(self.split_by_newline(&stdout))
    }

    fn split_by_newline<T: FromIterator<String>>(&self, text: &[u8]) -> T {
        text.split(|byte| *byte == b'\n')
            .filter(|bytes| !bytes.is_empty())
            .map(|bytes| String::from_utf8_lossy(bytes).to_string())
            .collect()
    }

    fn split_by_newline_vec(&self, text: &[u8]) -> Vec<String> {
        self.split_by_newline(text)
    }

    fn execute<I, S>(&self, args: I) -> io::Result<Output>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        Command::new("git").args(args).output()
    }

    fn execute_in_dir<I, S, P>(&self, args: I, dir: P) -> io::Result<Output>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
        P: AsRef<Path>,
    {
        Command::new("git").args(args).current_dir(dir).output()
    }

    fn execute_and_log<I, S>(&self, args: I)
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        match self.execute(args) {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let stderr = String::from_utf8_lossy(&output.stderr);

                if output.status.success() {
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
}
