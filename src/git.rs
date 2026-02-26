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
        let names: Vec<String> = self.split_by_newline(&output.stdout);

        Ok(names)
    }

    pub fn diff_staged_name_only(&self) -> io::Result<Vec<String>> {
        let output = self.execute(["diff", "--staged", "--name-only"])?;
        let names: Vec<String> = self.split_by_newline(&output.stdout);

        Ok(names)
    }

    pub fn pull(&self) -> Vec<String> {
        self.execute_returning_logs(["pull"])
    }

    fn remove_diff_headers(&self, stdout: &[u8]) -> String {
        static RE: LazyLock<Regex> =
            LazyLock::new(|| Regex::new(r"diff --git .*\nindex .*\n--- .*\n\+\+\+ .*\n").unwrap());
        let diff = String::from_utf8_lossy(stdout);

        RE.replace_all(&diff, "").to_string()
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
