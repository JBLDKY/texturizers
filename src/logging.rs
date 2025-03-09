#![warn(clippy::pedantic, clippy::nursery)]
// Prevent console window in addition to Slint window in Windows release builds when, e.g., starting the app via file manager. Ignored on other platforms.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use env_logger::Builder;
use log::LevelFilter;

pub fn setup_logging() {
    // let app_dir = home_dir().ok_or_else(|| anyhow!("Cannot find home directory"))?;

    Builder::new()
        // .target(env_logger::Target::Pipe(Box::new(log_file)))
        .filter_level(LevelFilter::Debug)
        .init();
}
