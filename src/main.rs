#![deny(clippy::panic, clippy::unwrap_used, clippy::expect_used, clippy::indexing_slicing)]

use std::collections::BTreeSet;

use anyhow::Context;

mod args;
use args::*;
mod typed_command;
use typed_command::*;
mod zfs_types;
use zfs_types::*;
mod organised_data;
use organised_data::*;
mod commands;
use commands::*;

fn get_first_and_last<T: std::cmp::Ord>(set: &BTreeSet<T>) -> Option<(&T, &T)> {
    Some((set.first()?, set.last()?))
}

/// Select a (from, to) local snapshot pair to sync to the remote that results in
/// the most recent snapshot on the local machine being transferred to the remote,
/// with an optimally small incremental stream.
///
/// The snapshot age invariant maintained by this tool between snapshots is that:
///   oldest_local <= youngest_common <= oldest_remote <= youngest_local
///
/// If this invariant is broken, then e.g. aggressive snapshot pruning or a long time
/// without syncing has broken the incremental chain, and manual intervention is needed.
fn snapshots_to_sync<'a>(
    local_dataset: &DatasetName,
    local_snapshots: &'a BTreeSet<OrganisedSnapshot>,
    remote_dataset: &DatasetName,
    remote_snapshots: &BTreeSet<OrganisedSnapshot>,
) -> anyhow::Result<Option<(&'a OrganisedSnapshot, &'a OrganisedSnapshot)>> {
    // Handle the empty local/remote snapshot cases.
    let Some((oldest_local, youngest_local)) = get_first_and_last(local_snapshots) else {
        // If there's no local snapshots, we don't need to copy anything.
        log_if_verbose!("SKIP: dataset {} has no local snapshots", local_dataset);
        return Ok(None);
    };
    if remote_snapshots.is_empty() {
        // No snapshots on the remote, send our entire history.
        log_if_verbose!(
            "SYNC ALL: remote dataset {} has no snapshots, send everything",
            remote_dataset
        );
        return Ok(Some((oldest_local, youngest_local)));
    };

    let Some((youngest_common_local, youngest_common_remote)) =
        youngest_common_ancestor(local_snapshots, remote_snapshots)
    else {
        anyhow::bail!(
            "no common snapshot between `{}` and `{}`: history has diverged, fail to be safe",
            local_dataset,
            remote_dataset
        )
    };

    // If the most recent shared snapshot is the latest local snapshot, the remote is already up-to-date.
    if youngest_common_local == youngest_local {
        log_if_verbose!(
            "SKIP: most recent snapshot `{}` is already present on remote as `{}`",
            youngest_local.full_name,
            youngest_common_remote.full_name,
        );
        Ok(None)
    } else {
        log_if_verbose!(
            "DELTA: should send from `{}` to `{}`",
            youngest_common_local.full_name,
            youngest_local.full_name,
        );
        Ok(Some((youngest_common_local, youngest_local)))
    }
}

fn sync_snapshots(
    local_dataset: &DatasetName,
    local_snapshots: &BTreeSet<OrganisedSnapshot>,
    remote_dataset: &DatasetName,
    remote_snapshots: &BTreeSet<OrganisedSnapshot>,
) -> anyhow::Result<()> {
    let Some((from, to)) = snapshots_to_sync(local_dataset, local_snapshots, remote_dataset, remote_snapshots)? else {
        return Ok(());
    };
    log!(
        "SEND: sending [{}, {}] to {}",
        from.full_name,
        to.full_name,
        remote_dataset
    );

    let mut command = PipedCommand::new(
        make_zfs_incremental_send_command(&from.full_name, &to.full_name),
        make_run_via_ssh_command(&ARGS.remote, make_zfs_recv_command(remote_dataset)),
    );
    command.run_or_dry_run()
}

fn main() -> anyhow::Result<()> {
    // Make sure command line args are parsed first.
    std::sync::LazyLock::force(&ARGS);

    let local: OrganisedSnapshots = make_zfs_list_snapshots_command(Some(&ARGS.source_dataset))
        .run()
        .context("failed to fetch snapshots from local")?
        .into();

    let all_datasets = make_run_via_ssh_command(&ARGS.remote, make_zfs_list_datasets_command())
        .run()
        .context("failed to fetch all datasets ")?;
    if !all_datasets.datasets.contains_key(&ARGS.remote_dataset) {
        log!(
            "CREATE: Remote dataset `{}` doesn't exist on remote, creating it.",
            &ARGS.remote_dataset
        );
        // We've been told to replicate to a dataset that doesn't exist: create it.
        make_run_via_ssh_command(&ARGS.remote, make_zfs_create_dataset_command(&ARGS.remote_dataset))
            .run_or_dry_run()?;
    }

    let remote: OrganisedSnapshots = make_run_via_ssh_command(
        &ARGS.remote,
        make_zfs_list_snapshots_command(Some(&ARGS.remote_dataset)),
    )
    .run()
    .context("failed to fetch snapshots from remote")?
    .into();

    for (local_dataset, local_snapshots) in local.datasets {
        let Some(suffix) = local_dataset.strip_prefix(&ARGS.source_dataset) else {
            // This dataset isn't under the tree we've been told to look at.
            continue;
        };

        let remote_dataset = format!("{}{}", &ARGS.remote_dataset, suffix);
        let remote_snapshots = if let Some(snaps) = remote.datasets.get(&remote_dataset) {
            snaps.clone()
        } else {
            // Dataset doesn't exist on remote - make it and return an empty snapshot list.
            let mut create_command =
                make_run_via_ssh_command(&ARGS.remote, make_zfs_create_dataset_command(&remote_dataset));
            create_command.run_or_dry_run()?;
            BTreeSet::new()
        };

        sync_snapshots(&local_dataset, &local_snapshots, &remote_dataset, &remote_snapshots)?;
    }

    Ok(())
}
