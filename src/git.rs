use regex::Regex;
use std::{
    ffi::OsStr,
    io::{self},
    process::{Command, Output},
    sync::LazyLock,
};

#[derive(Default)]
pub struct Git;

impl Git {
    pub fn diff(&self, path: &str) -> io::Result<String> {
        let output = self.execute(["diff", path])?;
        let diff = self.remove_diff_headers(&output.stdout);

        Ok(diff)
    }

    pub fn diff_staged(&self, path: &str) -> io::Result<String> {
        let output = self.execute(["diff", "--staged", path])?;
        let diff = self.remove_diff_headers(&output.stdout);

        Ok(diff)
    }

    pub fn diff_name_only(&self) -> io::Result<Vec<String>> {
        let output = self.execute(["diff", "--name-only"])?;
        let names: Vec<String> = self.split_by_newlines(&output.stdout);

        Ok(names)
    }

    pub fn diff_staged_name_only(&self) -> io::Result<Vec<String>> {
        let output = self.execute(["diff", "--staged", "--name-only"])?;
        let names: Vec<String> = self.split_by_newlines(&output.stdout);

        Ok(names)
    }

    pub fn pull(&self) -> io::Result<Output> {
        self.execute(["pull"])
    }

    fn remove_diff_headers(&self, stdout: &[u8]) -> String {
        static RE: LazyLock<Regex> =
            LazyLock::new(|| Regex::new(r"diff --git .*\nindex .*\n--- .*\n\+\+\+ .*\n").unwrap());
        let diff = String::from_utf8_lossy(stdout);

        RE.replace_all(&diff, "").to_string()
    }

    fn split_by_newlines(&self, stdout: &[u8]) -> Vec<String> {
        stdout
            .split(|byte| *byte == b'\n')
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
}
