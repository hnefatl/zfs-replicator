use std::ffi::OsString;

use crate::args::ARGS;
use crate::typed_command::*;
use crate::zfs_types::*;

pub fn make_zfs_list_snapshots_command(parent_dataset: Option<&DatasetName>) -> TypedCommand<ZfsListSnapshotOutput> {
    let mut c = TypedCommand::new("zfs");
    c.args(["list", "-t", "snapshot", "--json", "--json-int"]);

    if let Some(parent_dataset) = parent_dataset {
        // Recursive from the parent dataset down.
        c.args(["-r", parent_dataset]);
    }
    c
}
pub fn make_zfs_list_datasets_command() -> TypedCommand<ZfsListDatasetOutput> {
    let mut c = TypedCommand::new("zfs");
    c.args(["list", "-t", "filesystem", "--json"]);
    c
}
pub fn make_zfs_create_dataset_command(dataset: &DatasetName) -> TypedCommand<()> {
    let mut c = TypedCommand::new("zfs");
    c.args(["create", "-u", "-o", "readonly=on", dataset]);
    c
}
pub fn make_zfs_incremental_send_command(from: &SnapshotFullName, to: &SnapshotFullName) -> TypedCommand<Vec<u8>> {
    let mut c = TypedCommand::new("zfs");
    c.args(["send", "--replicate", "--raw"]);
    // Just send a single snapshot if there's just one snapshot in the range: otherwise send incremental between them.
    if from == to {
        c.arg(from);
    } else {
        c.args(["-I", from, to]);
    }
    c
}
pub fn make_zfs_recv_command(output_dataset: &DatasetName) -> TypedCommand<()> {
    let mut c = TypedCommand::new("zfs");
    c.args([
        "receive",
        "-u",
        "-s",
        "-o",
        "compress=off",
        "-o",
        "readonly=on",
        "-v",
        output_dataset,
    ]);
    c
}

pub fn make_run_via_ssh_command<T: serde::de::DeserializeOwned>(
    target: &str,
    nested_command: TypedCommand<T>,
) -> TypedCommand<T> {
    let mut c = TypedCommand::new("ssh");
    c.arg(target);
    if let Some(known_hosts_file) = &ARGS.known_hosts_file {
        let mut opt = OsString::from("UserKnownHostsFile=");
        opt.push(known_hosts_file.as_os_str());
        c.arg("-o");
        c.arg(opt);
    }
    if let Some(identity_file) = &ARGS.identity_file {
        c.arg("-i");
        c.arg(identity_file.as_os_str());
    }

    c.arg("--");
    c.arg(nested_command.get_program());
    c.args(nested_command.get_args());
    c
}
