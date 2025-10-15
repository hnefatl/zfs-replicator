use clap::Parser;
use std::{path::PathBuf, sync::LazyLock};

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

    /// File containing SSH known hosts.
    #[arg(long, default_value=None)]
    pub known_hosts_file: Option<PathBuf>,
    /// File containing SSH private key.
    #[arg(long, default_value=None)]
    pub identity_file: Option<PathBuf>,

    /// Print verbose tracelogs.
    #[arg(short, long, default_value_t = false)]
    pub verbose: bool,
    /// Dry-run ZFS mutating commands.
    #[arg(long, default_value_t = false)]
    pub dry_run: bool,
}

mod macros {
    #[macro_export]
    macro_rules! log {
        ($($arg:tt)*) => {
            println!($($arg)*);
        }
    }
    #[macro_export]
    macro_rules! log_if_verbose {
        ($($arg:tt)*) => {
            if ARGS.verbose {
                log!($($arg)*);
            }
        }
    }
}
