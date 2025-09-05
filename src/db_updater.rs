/* src/db_updater.rs */

use crate::config::Config;
use fancy_log::{LogLevel, log};
use maxminddb::Reader;
use parking_lot::RwLock;
use std::fs;
use std::io::copy;
use std::path::Path;
use std::sync::Arc;
use tokio::time::{Instant, sleep};

// Holds the state shared between the API and the updater task.
#[derive(Default)]
pub struct AppState {
    pub asn_db: Option<Reader<Vec<u8>>>,
    pub city_db: Option<Reader<Vec<u8>>>,
    pub country_db: Option<Reader<Vec<u8>>>,
}

/// Spawns a background task that periodically checks for and updates the MMDB files.
pub fn start_db_update_task(config: Config, state: Arc<RwLock<AppState>>) {
    tokio::spawn(async move {
        // Run once immediately on startup
        run_update_check(&config, &state).await;

        loop {
            sleep(config.update_interval).await;
            run_update_check(&config, &state).await;
        }
    });
}

/// Performs the database download and update logic.
async fn run_update_check(config: &Config, state: &Arc<RwLock<AppState>>) {
    log(LogLevel::Info, "Starting DB update check...");
    let start_time = Instant::now();

    // Ensure base and tmp directories exist
    if let Err(e) = fs::create_dir_all(&config.db_path) {
        log(
            LogLevel::Error,
            &format!(
                "Failed to create DB directory {}: {}",
                config.db_path.display(),
                e
            ),
        );
        return;
    }
    if let Err(e) = fs::create_dir_all(&config.tmp_path) {
        log(
            LogLevel::Error,
            &format!(
                "Failed to create tmp directory {}: {}",
                config.tmp_path.display(),
                e
            ),
        );
        return;
    }

    // Download and load each database
    let asn_path = config.db_path.join("GeoLite2-ASN.mmdb");
    let city_path = config.db_path.join("GeoLite2-City.mmdb");
    let country_path = config.db_path.join("GeoLite2-Country.mmdb");

    let new_asn = update_and_load_db("ASN", &config.asn_db_url, &asn_path, &config.tmp_path).await;
    let new_city =
        update_and_load_db("City", &config.city_db_url, &city_path, &config.tmp_path).await;
    let new_country = update_and_load_db(
        "Country",
        &config.country_db_url,
        &country_path,
        &config.tmp_path,
    )
    .await;

    // Atomically update the shared state with the new readers
    let mut state_guard = state.write();
    state_guard.asn_db = new_asn;
    state_guard.city_db = new_city;
    state_guard.country_db = new_country;

    log(
        LogLevel::Info,
        &format!("DB update check finished in {:?}", start_time.elapsed()),
    );
}

/// Handles downloading a single DB file if it's missing or an update is forced.
async fn update_and_load_db(
    name: &str,
    url: &str,
    final_path: &Path,
    tmp_path: &Path,
) -> Option<Reader<Vec<u8>>> {
    if !final_path.exists() {
        log(
            LogLevel::Info,
            &format!("{} database not found. Downloading from {}...", name, url),
        );
        if let Err(e) = download_db_file(url, final_path, tmp_path).await {
            log(
                LogLevel::Error,
                &format!("Failed to download {} database: {}", name, e),
            );
            return None;
        }
    }

    match Reader::open_readfile(final_path) {
        Ok(reader) => {
            log(
                LogLevel::Info,
                &format!("Successfully loaded {} database.", name),
            );
            Some(reader)
        }
        Err(e) => {
            log(
                LogLevel::Error,
                &format!(
                    "Failed to load {} database from {}: {}",
                    name,
                    final_path.display(),
                    e
                ),
            );
            None
        }
    }
}

/// Downloads a file to a temporary location and then moves it to the final destination.
async fn download_db_file(
    url: &str,
    final_path: &Path,
    tmp_path: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let response = reqwest::get(url).await?.error_for_status()?;
    let tmp_file_path = tmp_path.join(final_path.file_name().unwrap());

    let mut tmp_file = fs::File::create(&tmp_file_path)?;
    let mut content = std::io::Cursor::new(response.bytes().await?);
    copy(&mut content, &mut tmp_file)?;

    fs::rename(&tmp_file_path, final_path)?;
    Ok(())
}
