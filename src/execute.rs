use log::{error, info};
use std::{
    ffi::OsStr,
    path::Path,
    process::{Command, Output},
};

pub trait Execute {
    fn execute<I, S>(&self, args: I, dir: Option<&Path>) -> anyhow::Result<Output>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>;

    fn execute_in<I, S>(&self, args: I, dir: &Path) -> anyhow::Result<Output>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        self.execute(args, Some(dir))
    }

    fn execute_here<I, S>(&self, args: I) -> anyhow::Result<Output>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        self.execute(args, None)
    }

    fn execute_and_log<I, S>(&self, args: I, dir: Option<&Path>)
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        match self.execute(args, dir) {
            Ok(Output {
                status,
                stdout,
                stderr,
            }) => {
                let stdout = String::from_utf8_lossy(&stdout);
                let stderr = String::from_utf8_lossy(&stderr);

                if status.success() {
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

    fn execute_and_log_in<I, S>(&self, args: I, dir: &Path)
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        self.execute_and_log(args, Some(dir));
    }

    fn execute_and_log_here<I, S>(&self, args: I)
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        self.execute_and_log(args, None);
    }
}

#[derive(Default)]
pub struct GitExecutor;

impl Execute for GitExecutor {
    fn execute<I, S>(&self, args: I, dir: Option<&Path>) -> anyhow::Result<Output>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        let mut command = Command::new("git");
        command.args(args);

        if let Some(dir) = dir {
            command.current_dir(dir);
        }

        Ok(command.output()?)
    }
}
