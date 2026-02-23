use std::{
    ffi::OsStr,
    io::{self},
    process::{Command, Output},
};

#[derive(Default)]
pub struct Git;

impl Git {
    pub fn diff_name_only(&self) -> io::Result<Output> {
        self.execute(["diff", "--name-only"])
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
