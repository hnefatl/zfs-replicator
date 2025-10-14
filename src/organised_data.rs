use std::collections::{BTreeMap, BTreeSet};

use crate::zfs_types::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OrganisedSnapshot {
    pub snapshot_name: SnapshotName,
    pub full_name: SnapshotFullName,
    pub createtxg: u64,
}
impl PartialOrd for OrganisedSnapshot {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for OrganisedSnapshot {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.createtxg.cmp(&other.createtxg)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OrganisedSnapshots {
    pub datasets: BTreeMap<DatasetName, BTreeSet<OrganisedSnapshot>>,
}
impl From<ZfsListSnapshotOutput> for OrganisedSnapshots {
    fn from(value: ZfsListSnapshotOutput) -> Self {
        let mut datasets = BTreeMap::<DatasetName, BTreeSet<OrganisedSnapshot>>::new();
        for snapshot in value.datasets.values() {
            datasets
                .entry(snapshot.dataset.clone())
                .or_default()
                .insert(OrganisedSnapshot {
                    snapshot_name: snapshot.snapshot_name.clone(),
                    full_name: snapshot.name.clone(),
                    createtxg: snapshot.createtxg,
                });
        }
        OrganisedSnapshots { datasets }
    }
}

/// Return the most recent snapshot in `from` that's also present in `against`, if one exists.
/// This does an n^2 comparison to avoid relying on snapshot name ordering, only exact matches
/// between snapshot names.
pub fn youngest_common_ancestor<'a>(
    from: &'a BTreeSet<OrganisedSnapshot>,
    against: &'a BTreeSet<OrganisedSnapshot>,
) -> Option<&'a OrganisedSnapshot> {
    against
        .iter()
        .rev()
        .find(|&a| from.iter().rev().any(|f| a.snapshot_name == f.snapshot_name))
}
