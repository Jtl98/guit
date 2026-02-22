use std::{
    ffi::OsStr,
    process::{Child, Command},
};

#[derive(Default)]
pub struct Git {
    child: Option<Child>,
}

impl Git {
    pub fn pull(&mut self) {
        self.execute(["pull"]);
    }

    pub fn is_executing(&self) -> bool {
        self.child.is_some()
    }

    pub fn update(&mut self) {
        if let Some(child) = &mut self.child {
            if let Ok(Some(_)) = child.try_wait() {
                self.child = None;
            }
        }
    }

    fn execute<I, S>(&mut self, args: I)
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        if self.child.is_none() {
            match Command::new("git").args(args).spawn() {
                Ok(child) => self.child = Some(child),
                Err(error) => eprintln!("{}", error),
            }
        }
    }
}
