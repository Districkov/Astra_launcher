use std::fs;
use std::path::Path;

use sha2::{Digest, Sha256};
use serde::{Deserialize, Serialize};

use crate::logging::{log_error, log_info, log_warn};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileEntry {
    pub path: String,
    pub sha256: String,
    pub size: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Manifest {
    pub version: String,
    #[serde(default)]
    pub gta_version: String,
    pub files: Vec<FileEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckResult {
    pub valid: bool,
    pub missing: Vec<FileEntry>,
    pub outdated: Vec<FileEntry>,
    pub total_download_size: u64,
    /// `true` when verification was skipped because no manifest is configured
    /// or reachable. The on-disk client is then trusted as-is.
    #[serde(default)]
    pub skipped: bool,
}

/// Download the client manifest, or return `None` when file verification should
/// be skipped.
///
/// Verification is skipped (and the already-installed client trusted) when:
/// - `url` is empty (no manifest configured), or
/// - the manifest cannot be fetched (network error / server down).
///
/// This is the direct-connect heart of ASTRA: RAGE:MP's central infrastructure
/// may be offline, but a locally-installed client must still be launchable.
/// When `url` *is* configured AND reachable, the strict size+SHA256 verification
/// in [`verify_files`] still runs.
pub async fn fetch_manifest_or_skip(url: &str) -> Result<Option<Manifest>, String> {
    if url.is_empty() {
        log_info!("client_manifest_url not configured — skipping client file verification");
        return Ok(None);
    }
    match download_manifest(url).await {
        Ok(m) => Ok(Some(m)),
        Err(e) => {
            log_warn!("client manifest unavailable ({}), skipping verification", e);
            Ok(None)
        }
    }
}

pub async fn download_manifest(url: &str) -> Result<Manifest, String> {
    if url.is_empty() {
        return Err("Manifest URL not configured".to_string());
    }

    log_info!("Downloading manifest from {}", url);

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

    let resp = client
        .get(url)
        .send()
        .await
        .map_err(|e| format!("Failed to fetch manifest: {}", e))?;

    if !resp.status().is_success() {
        return Err(format!("Manifest server returned status {}", resp.status()));
    }

    let manifest: Manifest = resp
        .json()
        .await
        .map_err(|e| format!("Failed to parse manifest: {}", e))?;

    log_info!(
        "Manifest v{}: {} files, GTA version: {}",
        manifest.version,
        manifest.files.len(),
        manifest.gta_version
    );

    Ok(manifest)
}

pub fn verify_files(client_dir: &Path, manifest: &Manifest) -> CheckResult {
    let mut missing = Vec::new();
    let mut outdated = Vec::new();
    let mut total_download_size: u64 = 0;

    for entry in &manifest.files {
        let file_path = client_dir.join(&entry.path);

        if !file_path.exists() {
            log_info!("Missing: {}", entry.path);
            total_download_size += entry.size;
            missing.push(entry.clone());
            continue;
        }

        // Check file size first (fast check)
        if let Ok(meta) = fs::metadata(&file_path) {
            if meta.len() != entry.size {
                log_info!(
                    "Outdated (size mismatch): {} (local={} vs expected={})",
                    entry.path,
                    meta.len(),
                    entry.size
                );
                total_download_size += entry.size;
                outdated.push(entry.clone());
                continue;
            }
        }

        // Check SHA-256 hash
        match compute_sha256(&file_path) {
            Ok(hash) => {
                if hash != entry.sha256 {
                    log_info!(
                        "Outdated (hash mismatch): {} (local={} vs expected={})",
                        entry.path,
                        hash,
                        entry.sha256
                    );
                    total_download_size += entry.size;
                    outdated.push(entry.clone());
                }
            }
            Err(e) => {
                log_error!("Failed to hash {}: {}", entry.path, e);
                total_download_size += entry.size;
                outdated.push(entry.clone());
            }
        }
    }

    let valid = missing.is_empty() && outdated.is_empty();
    log_info!(
        "Verify result: valid={}, missing={}, outdated={}, total_download={} bytes",
        valid,
        missing.len(),
        outdated.len(),
        total_download_size
    );

    CheckResult {
        valid,
        missing,
        outdated,
        total_download_size,
        skipped: false,
    }
}

pub fn compute_sha256(path: &Path) -> Result<String, String> {
    let data = fs::read(path).map_err(|e| format!("Failed to read {}: {}", path.display(), e))?;
    let mut hasher = Sha256::new();
    hasher.update(&data);
    let result = hasher.finalize();
    Ok(format!("{:x}", result))
}
