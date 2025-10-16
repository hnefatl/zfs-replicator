use anyhow::Context;
use serde::de::DeserializeOwned;
use shell_quote::QuoteInto;
use std::{marker::PhantomData, process::Command};

use crate::args::ARGS;
use crate::{log, log_if_verbose};

pub trait OutputType: Sized {
    fn parse(output: Vec<u8>) -> anyhow::Result<Self>;
}

pub struct IgnoreOutput;
impl OutputType for IgnoreOutput {
    fn parse(_: Vec<u8>) -> anyhow::Result<Self> {
        Ok(Self)
    }
}

#[allow(dead_code)]
pub struct RawOutput {
    pub output: Vec<u8>,
}
impl OutputType for RawOutput {
    fn parse(output: Vec<u8>) -> anyhow::Result<Self> {
        Ok(Self { output })
    }
}

#[allow(dead_code)]
pub struct StringOutput {
    pub output: String,
}
impl OutputType for StringOutput {
    fn parse(output: Vec<u8>) -> anyhow::Result<Self> {
        Ok(Self {
            output: String::from_utf8(output)?,
        })
    }
}

pub struct ParseableOutput<T> {
    pub output: T,
}
impl<T: DeserializeOwned> OutputType for ParseableOutput<T> {
    fn parse(output: Vec<u8>) -> anyhow::Result<Self> {
        Ok(Self {
            output: serde_json::from_slice::<T>(&output)?,
        })
    }
}

pub trait Runnable<Output: OutputType> {
    /// Run the command and deserialise the output.
    fn run(&mut self) -> anyhow::Result<Output>;
}
pub trait DryRunnable {
    // Dry-run the command if the dry-run flag has been set, otherwise run it.
    fn run_or_dry_run(&mut self) -> anyhow::Result<()>;
}
/// Only commands without output can be "generalisably" dry-runnable, because () is the only type we
/// know we can construct. Other commands needing to be dry-runned should be explicitly dry-run with
/// a conditional at the callsite.
impl<T: Runnable<IgnoreOutput> + std::fmt::Display> DryRunnable for T {
    fn run_or_dry_run(&mut self) -> anyhow::Result<()> {
        if ARGS.dry_run {
            log!("DRY RUN: {}", self);
        } else {
            self.run()?;
        }
        Ok(())
    }
}

/// A `std::process::Command` along with a type hint about what data should be output.
pub struct TypedCommand<Output> {
    command: Command,
    t: PhantomData<Output>,
}
impl<Output: OutputType> TypedCommand<Output> {
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
}
impl<Output: OutputType> Runnable<Output> for TypedCommand<Output> {
    fn run(&mut self) -> anyhow::Result<Output> {
        log_if_verbose!("RUN: `{}`", self);

        let output = self.command.output()?;
        if !output.status.success() {
            anyhow::bail!(
                "running command failed with {:?}: `{}`\nStdout:\n{}\nStderr:\n{}",
                output.status.code(),
                self,
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr),
            );
        }
        Output::parse(output.stdout)
    }
}
impl<Output> std::fmt::Display for TypedCommand<Output> {
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

pub struct PipedCommand<Ot> {
    from: TypedCommand<RawOutput>,
    to: TypedCommand<Ot>,
}
impl<Ot> PipedCommand<Ot> {
    pub fn new(from: TypedCommand<RawOutput>, to: TypedCommand<Ot>) -> Self {
        PipedCommand { from, to }
    }
}
impl<Ot: OutputType> Runnable<Ot> for PipedCommand<Ot> {
    fn run(&mut self) -> anyhow::Result<Ot> {
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
impl<Tt> std::fmt::Display for PipedCommand<Tt> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{} | {}", self.from, self.to))
    }
}
