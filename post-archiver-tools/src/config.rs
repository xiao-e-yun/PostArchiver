use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(
    name = "post-archiver",
    version,
    about = "Maintenance tools for post-archiver archives"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Show the archive version and statistics
    Info {
        /// Archive directory
        #[arg(default_value = "archive")]
        path: PathBuf,
    },
    /// Upgrade the archive in place to the latest version
    Migrate {
        /// Archive directory
        #[arg(default_value = "archive")]
        path: PathBuf,
        /// Skip the confirmation prompt (required in non-interactive sessions)
        #[arg(short, long)]
        yes: bool,
    },
    /// Remove files and directories not referenced by the database, then VACUUM it
    Vacuum {
        /// Archive directory
        #[arg(default_value = "archive")]
        path: PathBuf,
        /// Only report what would be removed
        #[arg(long)]
        dry_run: bool,
    },
}

/// State shared by the migration steps.
#[derive(Debug, Clone)]
pub struct Config {
    /// Archive being migrated (in place).
    pub target: PathBuf,
    pub updated: bool,
}
