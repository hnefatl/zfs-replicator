use anyhow::Context;
use serde::de::DeserializeOwned;
use shell_quote::QuoteInto;
use std::{marker::PhantomData, process::Command};

use crate::args::ARGS;
use crate::{log, log_if_verbose};

/// A `std::process::Command` along with a type hint about what data should be output.
pub struct TypedCommand<T> {
    command: Command,
    t: PhantomData<T>,
}
impl<T: DeserializeOwned> TypedCommand<T> {
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

    pub fn run(&mut self) -> anyhow::Result<T> {
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
impl<T> std::fmt::Display for TypedCommand<T> {
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

pub struct PipedCommand<Tf, Tt> {
    from: TypedCommand<Tf>,
    to: TypedCommand<Tt>,
}
impl<Tf, Tt: DeserializeOwned> PipedCommand<Tf, Tt> {
    pub fn new(from: TypedCommand<Tf>, to: TypedCommand<Tt>) -> Self {
        PipedCommand { from, to }
    }
    pub fn run(&mut self) -> anyhow::Result<Tt> {
        log_if_verbose!("PIPE START: `{} | ...`", self.from);

        let mut s = self.from.command.stdout(std::process::Stdio::piped()).spawn()?;
        let s_stdout = s.stdout.take().context("failed to get child process stdout")?;
        self.to.command.stdin(std::process::Stdio::from(s_stdout));
        let result = self.to.run();
        // Make sure the sender process has terminated once the receiver finishes.
        s.kill()?;
        result
    }
}
impl<Tf, Tt> std::fmt::Display for PipedCommand<Tf, Tt> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{} | {}", self.from, self.to))
    }
}
