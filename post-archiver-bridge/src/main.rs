pub mod config;
mod v1;
mod v2;

use std::{fs, io, path::Path};

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

    if !config.overwrite {
        if config.target.exists() {
            error!("target is already exists");
        } else {
            copy_dir_all(&config.source, &config.target).unwrap();

            // https://stackoverflow.com/a/65192210/15859431
            fn copy_dir_all(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> io::Result<()> {
                fs::create_dir_all(&dst)?;
                for entry in fs::read_dir(src)? {
                    let entry = entry?;
                    let ty = entry.file_type()?;
                    if ty.is_dir() {
                        copy_dir_all(entry.path(), dst.as_ref().join(entry.file_name()))?;
                    } else {
                        fs::copy(entry.path(), dst.as_ref().join(entry.file_name()))?;
                    }
                }
                Ok(())
            }
        }}

    v1::Bridge::verify_and_upgrade(&mut config);
    v2::Bridge::verify_and_upgrade(&mut config);

    if config.updated {
        info!("Successfully updated");
    } else {
        info!("No updates required");
    }
}


pub trait Migration: Default {
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
