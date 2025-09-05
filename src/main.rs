/* src/main.rs */

mod api;
mod config;
mod db_updater;

use crate::config::Config;
use db_updater::AppState;
use dotenvy::dotenv;
use fancy_log::{LogLevel, log, set_log_level};
use lazy_motd::lazy_motd;
use parking_lot::RwLock;
use std::env;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // --- Initialization ---
    dotenv().ok();
    let level = env::var("LOG_LEVEL")
        .unwrap_or_else(|_| "info".to_string())
        .to_lowercase();
    let log_level = match level.as_str() {
        "debug" => LogLevel::Debug,
        "warn" => LogLevel::Warn,
        "error" => LogLevel::Error,
        _ => LogLevel::Info,
    };
    set_log_level(log_level);
    lazy_motd!();

    // --- Load Config and Prepare State ---
    let config = Config::from_env();
    log(
        LogLevel::Info,
        &format!("Socket path set to: {}", config.socket_path.display()),
    );
    log(
        LogLevel::Info,
        &format!("Database path set to: {}", config.db_path.display()),
    );

    // AppState holds the database readers, protected by a lock for safe concurrent access
    let app_state = Arc::new(RwLock::new(AppState::default()));

    // --- Start Background and Foreground Services ---
    // Start the background task for updating DB files
    db_updater::start_db_update_task(config.clone(), app_state.clone());

    // Start the API server in the foreground
    api::start_api_server(config, app_state).await?;

    Ok(())
}
