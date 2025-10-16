use std::collections::HashMap;

use serde::Deserialize;

// TODO: translate to `struct(String)` with custom validation on parsing.
// E.g. `zfast/enc/freqsnap@autosnap_2025-10-12_21:40:13_daily`
pub type SnapshotFullName = String;
// E.g. `autosnap_2025-10-12_21:40:13_daily`
pub type SnapshotName = String;
// E.g. `zfast/enc/freqsnap`
pub type DatasetName = String;

#[derive(Deserialize, Debug, Clone)]
pub struct Snapshot {
    pub name: SnapshotFullName,
    pub createtxg: u64,
    pub dataset: DatasetName,
    pub snapshot_name: SnapshotName,

    #[serde(rename = "type")]
    _t: monostate::MustBe!("SNAPSHOT"),
}

#[derive(Deserialize, Debug, Clone)]
pub struct Dataset {
    pub name: DatasetName,
    #[serde(rename = "type")]
    _t: monostate::MustBe!("FILESYSTEM"),
}

#[derive(Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum ZfsListItem {
    Snapshot(Snapshot),
    Dataset(Dataset),
}
impl ZfsListItem {
    pub fn dataset_name(&self) -> &DatasetName {
        match self {
            Self::Snapshot(snapshot) => &snapshot.dataset,
            Self::Dataset(dataset) => &dataset.name,
        }
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct ZfsListOutput {
    #[serde(rename = "datasets")]
    pub output: HashMap<String, ZfsListItem>,
}
