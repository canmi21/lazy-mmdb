/* src/api.rs */

use crate::config::Config;
use crate::db_updater::AppState;
use fancy_log::{LogLevel, log};
use lazy_sock::{Method, Request, Response, lazy_sock};
use parking_lot::RwLock;
use std::net::IpAddr;
use std::sync::Arc;

/// Starts the Unix Domain Socket API server.
pub async fn start_api_server(
    config: Config,
    state: Arc<RwLock<AppState>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let server = lazy_sock!(&config.socket_path);

    // --- Define API Routes ---

    let asn_state = state.clone();
    server
        .route(Method::Get, "/lookup/asn", move |req| {
            handle_asn_lookup(req, &asn_state)
        })
        .await;

    let city_state = state.clone();
    server
        .route(Method::Get, "/lookup/city", move |req| {
            handle_city_lookup(req, &city_state)
        })
        .await;

    let country_state = state.clone();
    server
        .route(Method::Get, "/lookup/country", move |req| {
            handle_country_lookup(req, &country_state)
        })
        .await;

    log(LogLevel::Info, "API server configured. Starting...");
    server.run().await
}

// Common logic to extract IP from request.
fn get_ip_from_request(req: &Request) -> Result<IpAddr, Response> {
    let ip_str = req
        .query_params()
        .get("ip")
        .ok_or_else(|| Response::new(400).with_text("Missing 'ip' query parameter"))?
        .to_string();

    ip_str
        .parse()
        .map_err(|_| Response::new(400).with_text("Invalid IP address format"))
}

// Specific handler for ASN lookups.
fn handle_asn_lookup(req: Request, state: &Arc<RwLock<AppState>>) -> Response {
    let ip = match get_ip_from_request(&req) {
        Ok(ip) => ip,
        Err(resp) => return resp,
    };

    let state_guard = state.read();
    match &state_guard.asn_db {
        Some(reader) => match reader.lookup::<maxminddb::geoip2::Asn>(ip) {
            Ok(Some(data)) => match serde_json::to_string(&data) {
                Ok(json) => Response::json(&json),
                Err(e) => Response::internal_error(&format!("Failed to serialize data: {}", e)),
            },
            Ok(None) => Response::not_found("IP address not found in the database"),
            Err(e) => Response::internal_error(&format!("Database lookup error: {}", e)),
        },
        None => Response::new(503).with_text("ASN database is not available."),
    }
}

// Specific handler for City lookups.
fn handle_city_lookup(req: Request, state: &Arc<RwLock<AppState>>) -> Response {
    let ip = match get_ip_from_request(&req) {
        Ok(ip) => ip,
        Err(resp) => return resp,
    };

    let state_guard = state.read();
    match &state_guard.city_db {
        Some(reader) => match reader.lookup::<maxminddb::geoip2::City>(ip) {
            Ok(Some(data)) => match serde_json::to_string(&data) {
                Ok(json) => Response::json(&json),
                Err(e) => Response::internal_error(&format!("Failed to serialize data: {}", e)),
            },
            Ok(None) => Response::not_found("IP address not found in the database"),
            Err(e) => Response::internal_error(&format!("Database lookup error: {}", e)),
        },
        None => Response::new(503).with_text("City database is not available."),
    }
}

// Specific handler for Country lookups.
fn handle_country_lookup(req: Request, state: &Arc<RwLock<AppState>>) -> Response {
    let ip = match get_ip_from_request(&req) {
        Ok(ip) => ip,
        Err(resp) => return resp,
    };

    let state_guard = state.read();
    match &state_guard.country_db {
        Some(reader) => match reader.lookup::<maxminddb::geoip2::Country>(ip) {
            Ok(Some(data)) => match serde_json::to_string(&data) {
                Ok(json) => Response::json(&json),
                Err(e) => Response::internal_error(&format!("Failed to serialize data: {}", e)),
            },
            Ok(None) => Response::not_found("IP address not found in the database"),
            Err(e) => Response::internal_error(&format!("Database lookup error: {}", e)),
        },
        None => Response::new(503).with_text("Country database is not available."),
    }
}
