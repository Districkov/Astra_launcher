use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use tauri::Emitter;

use crate::logging::{log_info, log_warn};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheInfo {
    pub version: String,
    pub download_url: String,
    #[serde(default)]
    pub size_bytes: u64,
    #[serde(default)]
    pub file_count: u64,
    #[serde(default)]
    pub full: bool,
}

fn cache_version_path(rage_dir: &Path) -> PathBuf {
    rage_dir.join("client_resources").join("cache_version.txt")
}

pub fn get_local_cache_version(rage_dir: &Path) -> String {
    let path = cache_version_path(rage_dir);
    fs::read_to_string(&path)
        .map(|s| s.trim().to_string())
        .unwrap_or_default()
}

pub fn set_local_cache_version(rage_dir: &Path, version: &str) {
    let path = cache_version_path(rage_dir);
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    let _ = fs::write(&path, version);
}

pub async fn check_cache_needed(rage_dir: &Path, cache_info_url: &str) -> Result<(bool, String, String, CacheInfo), String> {
    if cache_info_url.is_empty() {
        return Err("Cache info URL not configured".to_string());
    }

    let local_version = get_local_cache_version(rage_dir);
    log_info!("Local cache version: '{}'", local_version);

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

    let resp = client
        .get(cache_info_url)
        .send()
        .await
        .map_err(|e| format!("Failed to fetch cache info: {}", e))?;

    if !resp.status().is_success() {
        return Err(format!("Cache info server returned status {}", resp.status()));
    }

    let cache_info: CacheInfo = resp
        .json()
        .await
        .map_err(|e| format!("Failed to parse cache info: {}", e))?;

    let needed = local_version != cache_info.version;
    let reason = if needed {
        if local_version.is_empty() {
            "First time - cache not downloaded yet".to_string()
        } else {
            format!("Local '{}' != Server '{}'", local_version, cache_info.version)
        }
    } else {
        "Cache is up to date".to_string()
    };

    log_info!(
        "Cache check: needed={}, reason={}, server_version={}",
        needed,
        reason,
        cache_info.version
    );

    Ok((needed, local_version, cache_info.version.clone(), cache_info))
}

/// Like [`check_cache_needed`], but returns `None` (skip) instead of erroring
/// when `cache_info_url` is empty or unreachable, so an already-installed client
/// can still launch without the central cache endpoint.
///
/// This mirrors the manifest behaviour: RAGE:MP's infrastructure may be down,
/// but a locally-installed client (which fetches missing resources from the
/// specific server on join anyway) must remain launchable.
pub async fn check_cache_needed_or_skip(
    rage_dir: &Path,
    cache_info_url: &str,
) -> Result<Option<(bool, String, String, CacheInfo)>, String> {
    if cache_info_url.is_empty() {
        log_info!("cache_info_url not configured — skipping server cache check");
        return Ok(None);
    }
    match check_cache_needed(rage_dir, cache_info_url).await {
        Ok(v) => Ok(Some(v)),
        Err(e) => {
            log_warn!("cache info unavailable ({}), skipping cache check", e);
            Ok(None)
        }
    }
}

pub async fn download_cache(
    rage_dir: &Path,
    cache_info: &CacheInfo,
    window: &tauri::Window,
) -> Result<(), String> {
    let cache_dir = rage_dir.join("client_resources");
    fs::create_dir_all(&cache_dir)
        .map_err(|e| format!("Failed to create cache dir {:?}: {}", cache_dir, e))?;

    log_info!(
        "Downloading cache v{} from {}",
        cache_info.version,
        cache_info.download_url
    );

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(300))
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

    let response = client
        .get(&cache_info.download_url)
        .send()
        .await
        .map_err(|e| format!("Failed to connect: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("Server returned status {}", response.status()));
    }

    let total_size = response.content_length().unwrap_or(0);
    let mut downloaded: u64 = 0;

    // Download to temp file
    let zip_path = std::env::temp_dir().join("astra_cache.zip");
    let mut file = fs::File::create(&zip_path)
        .map_err(|e| format!("Failed to create temp file: {}", e))?;

    let mut stream = response.bytes_stream();

    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| format!("Read error: {}", e))?;
        file.write_all(&chunk)
            .map_err(|e| format!("Write error: {}", e))?;
        downloaded += chunk.len() as u64;

        let percent = if total_size > 0 {
            (downloaded as f64 / total_size as f64 * 100.0) as u32
        } else {
            0
        };

        let _ = window.emit(
            "cache-download-progress",
            serde_json::json!({
                "phase": "download",
                "downloaded": downloaded,
                "total": total_size,
                "percent": percent,
            }),
        );
    }

    drop(file);

    // Extract zip
    log_info!("Extracting cache to {:?}", cache_dir);

    let _ = window.emit(
        "cache-download-progress",
        serde_json::json!({
            "phase": "extract",
            "percent": 0,
        }),
    );

    let zip_file = fs::File::open(&zip_path)
        .map_err(|e| format!("Failed to open zip: {}", e))?;

    let mut archive = zip::ZipArchive::new(zip_file)
        .map_err(|e| format!("Failed to read zip: {}", e))?;

    let total_files = archive.len();
    let mut extracted = 0;

    for i in 0..total_files {
        let mut zip_file = archive
            .by_index(i)
            .map_err(|e| format!("Failed to read zip entry: {}", e))?;

        let outpath = match zip_file.enclosed_name() {
            Some(path) => cache_dir.join(path),
            None => continue,
        };

        if let Some(parent) = outpath.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create dir: {}", e))?;
        }

        if zip_file.is_dir() {
            continue;
        }

        let mut outfile = fs::File::create(&outpath)
            .map_err(|e| format!("Failed to create file: {}", e))?;
        std::io::copy(&mut zip_file, &mut outfile)
            .map_err(|e| format!("Failed to write file: {}", e))?;

        extracted += 1;
        let percent = (extracted as f64 / total_files as f64 * 100.0) as u32;

        let _ = window.emit(
            "cache-download-progress",
            serde_json::json!({
                "phase": "extract",
                "percent": percent,
                "extracted": extracted,
                "total_files": total_files,
            }),
        );
    }

    // Clean up temp file
    let _ = fs::remove_file(&zip_path);

    // Save version
    set_local_cache_version(rage_dir, &cache_info.version);

    log_info!(
        "Cache extracted: {} files to {:?}",
        extracted,
        cache_dir
    );

    Ok(())
}
