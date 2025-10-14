use crate::typed_command::*;
use crate::zfs_types::*;

pub fn make_zfs_list_snapshots_command(parent_dataset: Option<&DatasetName>) -> TypedCommand<ZfsListSnapshotOutput, true> {
    let mut c = TypedCommand::new("zfs");
    c.args(["list", "-t", "snapshot", "--json", "--json-int"]);

    if let Some(parent_dataset) = parent_dataset {
        // Recursive from the parent dataset down.
        c.args(["-r", parent_dataset]);
    }
    c
}
pub fn make_zfs_list_datasets_command() -> TypedCommand<ZfsListDatasetOutput, true> {
    let mut c = TypedCommand::new("zfs");
    c.args(["list", "-t", "filesystem", "--json"]);
    c
}
pub fn make_zfs_create_dataset_command(dataset: &DatasetName) -> TypedCommand<(), false> {
    let mut c = TypedCommand::new("zfs");
    c.args(["create", dataset]);
    c
}
pub fn make_run_via_ssh_command<T: serde::de::DeserializeOwned, const RO: bool>(
    target: &str,
    nested_command: TypedCommand<T, RO>,
) -> TypedCommand<T, RO> {
    let mut c = TypedCommand::new("ssh");
    c.args([target, "--"]);
    c.arg(nested_command.get_program());
    c.args(nested_command.get_args());
    c
}
