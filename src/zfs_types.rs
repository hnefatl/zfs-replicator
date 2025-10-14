use std::{collections::HashMap};

use serde::Deserialize;

// E.g. `zfast/enc/freqsnap@autosnap_2025-10-12_21:40:13_daily`
pub type SnapshotFullName = String;
// E.g. `autosnap_2025-10-12_21:40:13_daily`
pub type SnapshotName = String;
// E.g. `zfast/enc/freqsnap`
pub type DatasetName = String;

#[derive(Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Snapshot {
    pub name: SnapshotFullName,
    #[serde(rename = "type")]
    pub datasettype: DatasetType,
    pub createtxg: u64,
    pub dataset: DatasetName,
    pub snapshot_name: SnapshotName,
}
impl PartialOrd for Snapshot {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for Snapshot {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // Order first by dataset name (for nice "a", "a/b", "a/b/c", "a/d" ordering), then by creation order of snapshot within the dataset.
        self.dataset
            .cmp(&other.dataset)
            .then(self.createtxg.cmp(&other.createtxg))
    }
}

// By only implementing the snapshot value we reject anything that's not a snapshot.
#[derive(Deserialize, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum DatasetType {
    #[serde(rename = "SNAPSHOT")]
    Snapshot,
}

#[derive(Deserialize, Debug, Clone)]
pub struct ZfsListOutput {
    pub datasets: HashMap<SnapshotFullName, Snapshot>,
}
