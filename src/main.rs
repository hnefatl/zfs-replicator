#![deny(
    clippy::panic,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing
)]

mod args;

use anyhow::Context;
use args::*;
mod typed_command;
use typed_command::*;
mod zfs_types;
use zfs_types::*;

fn make_zfs_list_snapshot_command(
    parent_dataset: Option<&DatasetName>,
) -> TypedCommand<ZfsListOutput> {
    let mut c = TypedCommand::new("zfs");
    c.args(["list", "-t", "snapshot", "--json", "--json-int"]);

    if let Some(parent_dataset) = parent_dataset {
        // Recursive from the parent dataset down.
        c.args(["-r", parent_dataset]);
    }
    c
}
fn make_run_via_ssh_command<T: serde::de::DeserializeOwned>(
    target: &str,
    nested_command: TypedCommand<T>,
) -> TypedCommand<T> {
    let mut c = TypedCommand::new("ssh");
    c.args([target, "--"]);
    c.arg(nested_command.get_program());
    c.args(nested_command.get_args());
    c
}

fn main() -> anyhow::Result<()> {
    // Make sure command line args are parsed first.
    std::sync::LazyLock::force(&ARGS);

    let local_snapshots = make_zfs_list_snapshot_command(Some(&ARGS.source_dataset))
        .run_and_parse_stdout()
        .context("failed to fetch snapshots from local")?;

    let remote_snapshots = make_run_via_ssh_command(
        &ARGS.remote,
        make_zfs_list_snapshot_command(Some(&ARGS.remote_dataset)),
    )
    .run_and_parse_stdout()
    .context("failed to fetch snapshots from remote")?;

    println!("{:?}", local_snapshots);
    println!("{:?}", remote_snapshots);
    Ok(())
}
