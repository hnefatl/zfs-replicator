use anyhow::Context;
use serde::de::DeserializeOwned;
use shell_quote::QuoteInto;
use std::{marker::PhantomData, process::Command};

use crate::args::ARGS;
use crate::{log, log_if_verbose};

pub trait TypedCommandExt<T> {
    fn run(&mut self) -> anyhow::Result<T>;
}

/// A `std::process::Command` along with a type hint about what data should be output.
pub struct TypedCommand<T, const READONLY: bool> {
    command: Command,
    t: PhantomData<T>,
}
impl<T: DeserializeOwned, const READONLY: bool> TypedCommand<T, READONLY> {
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
        log_if_verbose!("RUN: `{}`", self);

        let output = self.command.output()?;
        if !output.status.success() {
            anyhow::bail!(
                "running command failed: `{}`: {:?}\n{}",
                self,
                output.status.code(),
                String::from_utf8_lossy(&output.stderr),
            );
        }
        Ok(serde_json::from_slice::<T>(&output.stdout)?)
    }
}

impl<S> TypedCommand<S, true> {
    pub fn pipe_into(&mut self, receiver: &mut TypedCommand<(), false>) -> anyhow::Result<()> {
        log_if_verbose!("RUN: `{} | {}`", self, receiver);

        let mut s = self.command.stdout(std::process::Stdio::piped()).spawn()?;
        let s_stdout = s.stdout.take().context("failed to get child process stdout")?;
        receiver.command.stdin(std::process::Stdio::from(s_stdout));
        receiver.run()?;
        // Make sure the sender process has terminated once the receiver finishes.
        s.kill()?;
        Ok(())
    }
}

impl<T: DeserializeOwned> TypedCommandExt<T> for TypedCommand<T, true> {
    fn run(&mut self) -> anyhow::Result<T> {
        self._run()
    }
}
// Enforce that mutating commands only return `()`. Otherwise in dry-run mode, we won't run the command, so there's no
// valid output we could return.
impl TypedCommandExt<()> for TypedCommand<(), false> {
    fn run(&mut self) -> anyhow::Result<()> {
        if ARGS.dry_run {
            log!("DRY RUN: would run `{}`", self);
            Ok(())
        } else {
            self._run()
        }
    }
}

impl<T, const READONLY: bool> std::fmt::Display for TypedCommand<T, READONLY> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut s = std::ffi::OsString::new();
        s.push(self.command.get_program());
        for arg in self.command.get_args() {
            s.push(" ");
            shell_quote::Sh::quote_into(arg, &mut s);
        }
        f.write_str(&String::from_utf8_lossy(s.as_encoded_bytes()))
    }
}
