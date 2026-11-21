use std::fs;
use std::io::Write;
use std::path::Path;

use futures_util::StreamExt;
use tauri::Emitter;

use crate::logging::{log_error, log_info};
use crate::manifest::{compute_sha256, FileEntry};

pub async fn download_files(
    client_dir: &Path,
    files: &[FileEntry],
    base_url: &str,
    window: &tauri::Window,
) -> Result<(), String> {
    if files.is_empty() {
        return Ok(());
    }

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(120))
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

    let total_bytes: u64 = files.iter().map(|f| f.size).sum();
    let mut downloaded_bytes: u64 = 0;

    for (i, file) in files.iter().enumerate() {
        let file_url = format!("{}/{}", base_url.trim_end_matches('/'), file.path);
        let file_path = client_dir.join(&file.path);

        log_info!("Downloading [{}/{}] {} from {}", i + 1, files.len(), file.path, file_url);

        // Create parent directories
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create dir {:?}: {}", parent, e))?;
        }

        // Download with retry
        let mut success = false;
        for attempt in 1..=2 {
            match download_single_file(&client, &file_url, &file_path, window, &file.path, downloaded_bytes, total_bytes).await {
                Ok(()) => {
                    // Verify hash after download
                    match compute_sha256(&file_path) {
                        Ok(hash) if hash == file.sha256 => {
                            success = true;
                            break;
                        }
                        Ok(hash) => {
                            log_error!(
                                "Hash mismatch for {} after download (got={}, expected={}), attempt {}/2",
                                file.path, hash, file.sha256, attempt
                            );
                            let _ = fs::remove_file(&file_path);
                        }
                        Err(e) => {
                            log_error!("Failed to verify hash for {}: {}", file.path, e);
                            let _ = fs::remove_file(&file_path);
                        }
                    }
                }
                Err(e) => {
                    log_error!("Download attempt {}/2 failed for {}: {}", attempt, file.path, e);
                    let _ = fs::remove_file(&file_path);
                }
            }
        }

        if !success {
            return Err(format!("Failed to download {} after 2 attempts", file.path));
        }

        downloaded_bytes += file.size;
    }

    log_info!("All {} files downloaded successfully", files.len());
    Ok(())
}

async fn download_single_file(
    client: &reqwest::Client,
    url: &str,
    dest: &Path,
    window: &tauri::Window,
    file_name: &str,
    base_downloaded: u64,
    total_bytes: u64,
) -> Result<(), String> {
    let response = client
        .get(url)
        .send()
        .await
        .map_err(|e| format!("Connection error: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("HTTP {}", response.status()));
    }

    let mut file = fs::File::create(dest)
        .map_err(|e| format!("Failed to create file {:?}: {}", dest, e))?;

    let mut stream = response.bytes_stream();
    let mut file_downloaded: u64 = 0;

    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| format!("Read error: {}", e))?;
        file.write_all(&chunk)
            .map_err(|e| format!("Write error: {}", e))?;
        file_downloaded += chunk.len() as u64;

        let total_downloaded = base_downloaded + file_downloaded;
        let percent = if total_bytes > 0 {
            (total_downloaded as f64 / total_bytes as f64 * 100.0) as u32
        } else {
            0
        };

        let _ = window.emit(
            "client-download-progress",
            serde_json::json!({
                "downloaded": total_downloaded,
                "total": total_bytes,
                "percent": percent,
                "current_file": file_name,
            }),
        );
    }

    Ok(())
}
