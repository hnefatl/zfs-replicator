use std::{
    marker::PhantomData,
    ops::{Deref, DerefMut},
    process::Command,
};

use crate::args;

/// A `std::process::Command` along with a type hint about what data should be output.
pub struct TypedCommand<T> {
    command: Command,
    t: PhantomData<T>,
}
impl<T: serde::de::DeserializeOwned> TypedCommand<T> {
    pub fn new<S: AsRef<std::ffi::OsStr>>(program: S) -> Self {
        Self {
            command: std::process::Command::new(program),
            t: PhantomData,
        }
    }

    pub fn run_and_parse_stdout(mut self) -> anyhow::Result<T> {
        if args::ARGS.verbose {
            println!("Running command: {:?}", self.command)
        }

        let output = self.command.output()?;
        if !output.status.success() {
            anyhow::bail!(
                "running command failed: `{:?}`: {:?}\n{}",
                self.command,
                output.status.code(),
                String::from_utf8_lossy(&output.stderr),
            );
        }
        Ok(serde_json::from_slice::<T>(&output.stdout)?)
    }
}
impl<T> Deref for TypedCommand<T> {
    type Target = Command;
    fn deref(&self) -> &Self::Target {
        &self.command
    }
}
impl<T> DerefMut for TypedCommand<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.command
    }
}
