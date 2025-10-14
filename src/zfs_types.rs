use std::collections::HashMap;

use serde::Deserialize;

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
pub struct ZfsListSnapshotOutput {
    pub datasets: HashMap<SnapshotFullName, Snapshot>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Dataset {
    #[serde(rename = "type")]
    _t: monostate::MustBe!("FILESYSTEM"),
}

#[derive(Deserialize, Debug, Clone)]
pub struct ZfsListDatasetOutput {
    pub datasets: HashMap<DatasetName, Dataset>,
}
