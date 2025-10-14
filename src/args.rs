use clap::Parser;
use std::sync::LazyLock;

use crate::zfs_types::*;

// Global variable for easier access throughout the CLI.
pub static ARGS: LazyLock<Args> = LazyLock::new(Args::parse);

#[derive(Parser, Debug)]
#[command(rename_all = "snake_case")]
pub struct Args {
    /// Remote to send to, compatible with SSH naming (e.g. IP/hostname/...).
    #[arg(long)]
    pub remote: String,
    /// Dataset to replicate snapshots from.
    #[arg(long)]
    pub source_dataset: DatasetName,
    /// Dataset to replicate snapshots to.
    #[arg(long)]
    pub remote_dataset: DatasetName,

    /// Print verbose tracelogs.
    #[arg(short, long, default_value_t = false)]
    pub verbose: bool,
}
