pub mod config;
mod v1;

use std::fs;

use clap::Parser;
use config::Config;
use log::{info, warn, error};

fn main() {
    simple_logger::init().unwrap();
    info!("Starting up");

    let mut config = Config::parse();

    info!("");
    info!("- Configuration -----------------");
    info!("Source: {:?}", config.source);
    info!("Output: {:?}", config.target);
    info!("---------------------------------");
    info!("");

    if config.source == config.target {
        config.overwrite = true;
        warn!("Source and Target are the same");
        warn!("It will overwrite the source");
        info!("");
    }

    if !config.source.is_dir() {
        error!("Source does not exist");
        return;
    }

    if !config.target.exists() {
        warn!("Creating target directory");
        fs::create_dir_all(&config.target).expect("Unable to create target directory");
        info!("");
    }

    v1::BridgeV1::verify_and_upgrade(&mut config);

    if config.updated {
        info!("Successfully updated");
    } else {
        info!("No updates required");
    }
}

pub trait Bridge: Default {
    const VERSION: &'static str;
    fn verify(&mut self, config: &Config) -> bool;
    fn upgrade(&mut self, config: &mut Config);
    fn verify_and_upgrade(config: &mut Config) {
        let mut bridge = Self::default();

        info!("Checking for {} data", Self::VERSION);
        if bridge.verify(config) {
            info!("=================================");
            info!("Detected updating from {}", Self::VERSION);
            bridge.upgrade(config);
            config.source = config.target.clone();
            config.overwrite = true;
            config.updated = true;
            info!("=================================");
        }
        info!("");
    }
}
