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

fn sync_snapshots(
    local_dataset: &DatasetName,
    local_snapshots: &BTreeSet<OrganisedSnapshot>,
    remote_dataset: &DatasetName,
    remote_snapshots: &BTreeSet<OrganisedSnapshot>,
) -> anyhow::Result<()> {
    let Some(youngest_common_snapshot) = youngest_common_ancestor(local_snapshots, remote_snapshots) else {
        anyhow::bail!(
            "no common snapshot between `{}` and `{}`: history has diverged, fail to be safe",
            local_dataset,
            remote_dataset
        )
    };

    println!("Local dataset: {}", local_dataset);
    println!("  Oldest snapshot: {:?}", local_snapshots.last().map(|s| &s.full_name));
    println!(
        "  Youngest shared snapshot with remote: {:?}",
        youngest_common_snapshot.full_name
    );
    println!();
    Ok(())
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
        println!(
            "Remote dataset `{}` doesn't exist on remote, creating it.",
            &ARGS.remote_dataset
        );
        // We've been told to replicate to a dataset that doesn't exist: create it.
        make_run_via_ssh_command(&ARGS.remote, make_zfs_create_dataset_command(&ARGS.remote_dataset)).run()?;
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
            if !ARGS.dry_run {
                make_zfs_create_dataset_command(&remote_dataset).run()?;
            }
            BTreeSet::new()
        };

        sync_snapshots(&local_dataset, &local_snapshots, &remote_dataset, &remote_snapshots)?;
    }

    Ok(())
}
