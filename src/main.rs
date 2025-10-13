mod typed_command;
use typed_command::*;
mod zfs_snapshots;
use zfs_snapshots::*;

fn make_zfs_list_snapshot_command() -> TypedCommand<ZfsListOutput> {
    let mut c = TypedCommand::new("zfs");
    c.args(["list", "-t", "snapshot", "--json", "--json-int"]);
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

fn main() {
    let local_snapshots = make_zfs_list_snapshot_command()
        .run_and_parse_stdout()
        .expect("failed to fetch snapshots from local");

    let remote_snapshots = make_run_via_ssh_command("warthog", make_zfs_list_snapshot_command())
        .run_and_parse_stdout()
        .expect("failed to fetch snapshots from remote");

    println!("{:?}", local_snapshots);
    println!("{:?}", remote_snapshots);
}
