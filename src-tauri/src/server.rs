use std::net::TcpStream;
use std::time::{Duration, Instant};

use crate::config::LauncherConfig;
use crate::logging::{log_error, log_info, log_warn};

/// Timeout for each individual HTTP stats request.
///
/// The previous value was 5s **per request** and the two requests ran
/// sequentially, so a dead stats endpoint cost 10s — longer than the 10s poll
/// interval, which made polls overlap and look like an endless "checking…".
const HTTP_TIMEOUT: Duration = Duration::from_secs(3);

/// Timeout for the TCP reachability probe.
const TCP_TIMEOUT: Duration = Duration::from_secs(3);

/// How a server was judged to be online.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Reachability {
    /// HTTP stats answered.
    Http,
    /// No HTTP stats, but the game port accepted a TCP connection.
    Tcp,
    /// Neither worked.
    None,
}

impl Reachability {
    fn as_str(&self) -> &'static str {
        match self {
            Reachability::Http => "http",
            Reachability::Tcp => "tcp",
            Reachability::None => "none",
        }
    }
}

/// Probe the game port with a plain TCP connect.
///
/// This matters because a RAGE:MP server does not necessarily serve the
/// Server stats endpoints (`info.json` / `players.json`). If the port is
/// open the server process is alive, which is all the launcher really needs to
/// know before letting the user press Play.
async fn tcp_probe(address: &str) -> bool {
    let addr = address.to_string();
    tokio::task::spawn_blocking(move || {
        let Ok(sock_addr) = addr.parse() else {
            return false;
        };
        TcpStream::connect_timeout(&sock_addr, TCP_TIMEOUT).is_ok()
    })
    .await
    .unwrap_or(false)
}

/// Fetch a JSON document, returning `None` on any failure.
async fn fetch_json(client: &reqwest::Client, url: &str) -> Option<serde_json::Value> {
    match client.get(url).send().await {
        Ok(resp) => resp.json::<serde_json::Value>().await.ok(),
        Err(e) => {
            log_warn!("{} failed: {}", url, e);
            None
        }
    }
}

pub async fn get_server_status(config: &LauncherConfig) -> Result<serde_json::Value, String> {
    log_info!("get_server_status: querying {}", config.server_address);

    let client = reqwest::Client::builder()
        .timeout(HTTP_TIMEOUT)
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

    let base = format!("http://{}", config.server_address);
    let info_url = format!("{}/info.json", base);
    let players_url = format!("{}/players.json", base);

    // Run both requests concurrently — sequentially they cost 2× the timeout,
    // which is what pushed the poll interval over the edge.
    let (info, players) = tokio::join!(
        fetch_json(&client, &info_url),
        fetch_json(&client, &players_url),
    );

    let players_count = players
        .as_ref()
        .and_then(|p| p.as_array())
        .map(|arr| arr.len())
        .unwrap_or(0);

    let max_players = info
        .as_ref()
        .and_then(|i| i.get("vars"))
        .and_then(|v| v.get("sv_maxClients"))
        .and_then(|m| m.as_u64())
        .unwrap_or(0);

    // HTTP stats are nice-to-have; the TCP probe is what actually decides
    // reachability when the server does not expose them.
    let (reachability, ping_ms) = if info.is_some() {
        (Reachability::Http, 0u64)
    } else {
        let started = Instant::now();
        if tcp_probe(&config.server_address).await {
            (Reachability::Tcp, started.elapsed().as_millis() as u64)
        } else {
            (Reachability::None, 0)
        }
    };

    let online = reachability != Reachability::None;

    if !online {
        log_error!(
            "server {} is unreachable over both HTTP and TCP",
            config.server_address
        );
    }

    Ok(serde_json::json!({
        "online": online,
        "players": players_count,
        "max_players": max_players,
        "info": info,
        // Extra context so the UI can explain *why* it is (not) online.
        "reachability": reachability.as_str(),
        "has_stats": info.is_some(),
        "ping": ping_ms,
    }))
}

pub async fn get_server_ping(config: &LauncherConfig) -> Result<serde_json::Value, String> {
    let addr = config.server_address.clone();
    let start = Instant::now();

    let ok = tokio::task::spawn_blocking(move || {
        let Ok(sock_addr) = addr.parse() else {
            return false;
        };
        TcpStream::connect_timeout(&sock_addr, TCP_TIMEOUT).is_ok()
    })
    .await
    .unwrap_or(false);

    if ok {
        Ok(serde_json::json!({ "ping": start.elapsed().as_millis() as u64 }))
    } else {
        Ok(serde_json::json!({ "ping": 0, "error": "unreachable" }))
    }
}
