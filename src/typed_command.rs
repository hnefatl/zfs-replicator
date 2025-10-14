use std::{marker::PhantomData, process::Command};

use crate::args;

pub trait TypedCommandExt<T> {
    fn run(&mut self) -> anyhow::Result<T>;
}

/// A `std::process::Command` along with a type hint about what data should be output.
pub struct TypedCommand<T, const READONLY: bool> {
    command: Command,
    t: PhantomData<T>,
}
impl<T: serde::de::DeserializeOwned, const READONLY: bool> TypedCommand<T, READONLY> {
    pub fn new<S: AsRef<std::ffi::OsStr>>(program: S) -> Self {
        Self {
            command: std::process::Command::new(program),
            t: PhantomData,
        }
    }

    pub fn arg<S: AsRef<std::ffi::OsStr>>(&mut self, arg: S) -> &mut Command {
        self.command.arg(arg)
    }
    pub fn args<I, S>(&mut self, args: I) -> &mut Command
    where
        I: IntoIterator<Item = S>,
        S: AsRef<std::ffi::OsStr>,
    {
        self.command.args(args)
    }

    pub fn get_program(&self) -> &std::ffi::OsStr {
        self.command.get_program()
    }
    pub fn get_args(&self) -> std::process::CommandArgs<'_> {
        self.command.get_args()
    }

    fn _run(&mut self) -> anyhow::Result<T> {
        if args::ARGS.verbose {
            println!("RUN: `{:?}`", self.command)
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

impl<T: serde::de::DeserializeOwned> TypedCommandExt<T> for TypedCommand<T, true> {
    fn run(&mut self) -> anyhow::Result<T> {
        self._run()
    }
}
// Enforce that mutating commands only return `()`. Otherwise in dry-run mode, we won't run the command, so there's no
// valid output we could return.
impl TypedCommandExt<()> for TypedCommand<(), false> {
    fn run(&mut self) -> anyhow::Result<()> {
        if args::ARGS.dry_run {
            println!("DRY RUN: would run `{:?}`", self.command);
            Ok(())
        } else {
            self._run()
        }
    }
}
