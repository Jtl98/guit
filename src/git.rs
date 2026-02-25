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
    pub fn diff(&self, path: &str) -> io::Result<Output> {
        self.execute(["diff", path])
    }

    pub fn diff_staged(&self, path: &str) -> io::Result<Output> {
        self.execute(["diff", "--staged", path])
    }

    pub fn remove_diff_header(&self, output: Output) -> String {
        static RE: LazyLock<Regex> =
            LazyLock::new(|| Regex::new(r"diff --git .*\nindex .*\n--- .*\n\+\+\+ .*\n").unwrap());
        let stdout = String::from_utf8_lossy(&output.stdout);

        RE.replace_all(&stdout, "").to_string()
    }

    pub fn diff_name_only(&self) -> io::Result<Output> {
        self.execute(["diff", "--name-only"])
    }

    pub fn diff_staged_name_only(&self) -> io::Result<Output> {
        self.execute(["diff", "--staged", "--name-only"])
    }

    pub fn pull(&self) -> io::Result<Output> {
        self.execute(["pull"])
    }

    fn execute<I, S>(&self, args: I) -> io::Result<Output>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        Command::new("git").args(args).output()
    }
}
