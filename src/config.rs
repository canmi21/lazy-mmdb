/* src/config.rs */

use std::env;
use std::path::PathBuf;
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct Config {
    pub socket_path: PathBuf,
    pub db_path: PathBuf,
    pub tmp_path: PathBuf,
    pub update_interval: Duration,
    pub asn_db_url: String,
    pub city_db_url: String,
    pub country_db_url: String,
}

impl Config {
    /// Loads configuration from environment variables with hardcoded defaults.
    pub fn from_env() -> Self {
        let default_db_path = dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("/tmp"))
            .join("lazy-mmdb");

        let db_path = env::var("DB_PATH")
            .map(PathBuf::from)
            .unwrap_or(default_db_path);

        let socket_path = env::var("SOCKET_PATH")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("/tmp/lazy-mmdb/lazy-mmdb.sock"));

        let update_hours: u64 = env::var("UPDATE_INTERVAL_HOURS")
            .unwrap_or_else(|_| "24".to_string())
            .parse()
            .unwrap_or(24);

        Self {
            socket_path,
            tmp_path: db_path.join("tmp"),
            db_path,
            update_interval: Duration::from_secs(update_hours * 3600),
            asn_db_url: env::var("ASN_DB_URL").unwrap_or_else(|_| {
                "https://github.com/P3TERX/GeoLite.mmdb/raw/download/GeoLite2-ASN.mmdb".to_string()
            }),
            city_db_url: env::var("CITY_DB_URL").unwrap_or_else(|_| {
                "https://github.com/P3TERX/GeoLite.mmdb/raw/download/GeoLite2-City.mmdb".to_string()
            }),
            country_db_url: env::var("COUNTRY_DB_URL").unwrap_or_else(|_| {
                "https://github.com/P3TERX/GeoLite.mmdb/raw/download/GeoLite2-Country.mmdb"
                    .to_string()
            }),
        }
    }
}
