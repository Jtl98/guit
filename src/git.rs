use crate::diff::{DiffArea, DiffKey};
use regex::Regex;
use std::{
    ffi::OsStr,
    fs,
    io::{self},
    process::{Command, Output},
    sync::LazyLock,
};

#[derive(Default)]
pub struct Git;

impl Git {
    pub fn diff(&self, key: &DiffKey) -> io::Result<String> {
        static RE: LazyLock<Regex> =
            LazyLock::new(|| Regex::new(r"diff --git .*\nindex .*\n--- .*\n\+\+\+ .*\n").unwrap());

        let bytes = match key.area {
            DiffArea::Untracked => fs::read(&key.path)?,
            DiffArea::Unstaged => self.execute(["diff", &key.path])?.stdout,
            DiffArea::Staged => self.execute(["diff", "--staged", &key.path])?.stdout,
        };
        let diff = String::from_utf8_lossy(&bytes);

        Ok(RE.replace_all(&diff, "").to_string())
    }

    pub fn diff_name_only(&self) -> io::Result<Vec<DiffKey>> {
        let create_keys = |stdout, area| -> Vec<DiffKey> {
            self.split_by_newline(stdout)
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

    pub fn pull(&self) -> Vec<String> {
        self.execute_returning_logs(["pull"])
    }

    fn split_by_newline(&self, text: &[u8]) -> Vec<String> {
        text.split(|byte| *byte == b'\n')
            .filter(|bytes| !bytes.is_empty())
            .map(|bytes| String::from_utf8_lossy(bytes).to_string())
            .collect()
    }

    fn execute<I, S>(&self, args: I) -> io::Result<Output>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        Command::new("git").args(args).output()
    }

    fn execute_returning_logs<I, S>(&self, args: I) -> Vec<String>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        match self.execute(args) {
            Ok(output) => {
                let Output { stdout, stderr, .. } = output;
                let mut logs = self.split_by_newline(&stdout);
                logs.extend(self.split_by_newline(&stderr));

                logs
            }
            Err(error) => vec![error.to_string()],
        }
    }
}
