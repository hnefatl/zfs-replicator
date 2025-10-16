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
impl From<ZfsListOutput> for OrganisedSnapshots {
    fn from(value: ZfsListOutput) -> Self {
        let mut datasets = BTreeMap::<DatasetName, BTreeSet<OrganisedSnapshot>>::new();
        for item in value.output.values() {
            let dataset = datasets.entry(item.dataset_name().clone()).or_default();
            if let ZfsListItem::Snapshot(snapshot) = item {
                dataset.insert(OrganisedSnapshot {
                    snapshot_name: snapshot.snapshot_name.clone(),
                    full_name: snapshot.name.clone(),
                    createtxg: snapshot.createtxg,
                });
            };
        }
        OrganisedSnapshots { datasets }
    }
}

/// Return the most recent snapshot in `from` that's also present in `against`, if one exists.
/// This does an n^2 comparison to avoid relying on snapshot name ordering, only exact matches
/// between snapshot names.
pub fn youngest_common_ancestor<'f, 'a>(
    from: &'f BTreeSet<OrganisedSnapshot>,
    against: &'a BTreeSet<OrganisedSnapshot>,
) -> Option<(&'f OrganisedSnapshot, &'a OrganisedSnapshot)> {
    for a in against.iter().rev() {
        for f in from.iter().rev() {
            if f.snapshot_name == a.snapshot_name {
                return Some((f, a));
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn youngest_common_ancestor() {
        // TODO: unit tests.
        assert_eq!(4, 4);
    }
}
