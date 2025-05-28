use std::path::PathBuf;

use clap::Parser;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Parser)]
pub struct Config {
    #[clap(default_value = "./archiver")]
    pub source: PathBuf,

    #[clap(default_value = "./archiver-bridge")]
    pub target: PathBuf,

    #[clap(skip)]
    pub updated: bool,

    #[clap(skip)]
    pub overwrite: bool,
}
