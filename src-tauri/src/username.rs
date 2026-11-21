use std::fs;
use std::process::Command;

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

use crate::logging::log_info;
use crate::paths::cached_username_path;

pub fn get_username() -> Result<String, String> {
    // 1. Check cached name
    let cached = cached_username_path();
    if let Ok(name) = fs::read_to_string(&cached) {
        let trimmed = name.trim().to_string();
        if !trimmed.is_empty() {
            return Ok(trimmed);
        }
    }

    // 2. Try RAGE:MP registry
    let username = get_username_from_registry()
        .or_else(|| get_username_from_config())
        .unwrap_or_else(|| "Player".to_string());

    // Cache it
    let _ = fs::write(&cached, &username);

    Ok(username)
}

#[cfg(target_os = "windows")]
fn get_username_from_registry() -> Option<String> {
    // Try RAGE:MP registry key
    let output = Command::new("reg")
        .args([
            "query",
            r"HKCU\Software\RAGE-MP",
            "/v",
            "name",
        ])
        .creation_flags(0x00000008)
        .output()
        .ok()?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    for line in stdout.lines() {
        if line.contains("REG_SZ") {
            let parts: Vec<&str> = line.split("REG_SZ").collect();
            if parts.len() > 1 {
                let name = parts[1].trim().to_string();
                if !name.is_empty() {
                    log_info!("Username from registry: {}", name);
                    return Some(name);
                }
            }
        }
    }


    None
}

#[cfg(not(target_os = "windows"))]
fn get_username_from_registry() -> Option<String> {
    None
}

fn get_username_from_config() -> Option<String> {
    let local_appdata = dirs::data_local_dir()?;

    // Try RAGE:MP config
    let rage_config = local_appdata
        .join("RageMP")
        .join("settings.xml");
    if let Ok(content) = fs::read_to_string(&rage_config) {
        if let Some(name) = extract_name_from_xml(&content) {
            log_info!("Username from RageMP config: {}", name);
            return Some(name);
        }
    }

    // Try common.cfg
    let cfg_path = local_appdata
        .join("RageMP")
        .join("common.cfg");
    if let Ok(content) = fs::read_to_string(&cfg_path) {
        for line in content.lines() {
            if line.starts_with("set name ") {
                let name = line
                    .trim_start_matches("set name ")
                    .trim()
                    .trim_matches('"')
                    .to_string();
                if !name.is_empty() {
                    log_info!("Username from common.cfg: {}", name);
                    return Some(name);
                }
            }
        }
    }

    None
}

fn extract_name_from_xml(content: &str) -> Option<String> {
    if let Some(start) = content.find("<Name>") {
        if let Some(end) = content.find("</Name>") {
            let name = content[start + 6..end].trim().to_string();
            if !name.is_empty() {
                return Some(name);
            }
        }
    }
    None
}

pub fn save_username(name: &str) -> Result<(), String> {
    let path = cached_username_path();
    fs::write(&path, name).map_err(|e| format!("Failed to save username: {}", e))
}
