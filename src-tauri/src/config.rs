use std::fs;

use serde::{Deserialize, Serialize};

use crate::logging::{log_info, log_warn};
use crate::paths::launcher_data_dir;
use crate::patcher::Patch;

const DEFAULT_SERVER_ADDRESS: &str = "89.35.130.191:22005";
const DEFAULT_CLIENT_MANIFEST_URL: &str = "";
const DEFAULT_CACHE_INFO_URL: &str = "";
const DEFAULT_PATCHES_URL: &str = "";
const DEFAULT_FILE_DOWNLOAD_BASE_URL: &str = "";
const DEFAULT_RAGEMP_DOWNLOAD_URL: &str = "";
/// `"raw"` (rui auto-connect by raw IP) or `"hosts"` (whitelisted domain).
const DEFAULT_CONNECT_MODE: &str = "hosts";
const DEFAULT_HOSTS_DOMAIN: &str = "";
/// `"updater"` (official path) or `"ragemp_v"` (skip the updater + CDN check).
const DEFAULT_LAUNCH_TARGET: &str = "updater";
/// Inject the rui auto-connect patch? **Off by default, and off for a reason.**
///
/// Every attempt to edit `rui\index.rui` makes this client build refuse to load
/// the launcher UI at all — the CEF window goes black with "404 error: File not
/// found" and `_rageInitialize is not defined` in `cef_launcher_logs.txt`. Five
/// variants were tried (growing `app.js`, equal-length `app.js`, delayed call,
/// preserving the `getServers` handshake, and an `index.html`-only edit); all
/// were byte-correct in size, XXH64 hashes and JS syntax, and all broke it.
/// Factory `index.rui` works, so the client clearly validates the archive by a
/// mechanism not present in the file itself.
///
/// Leave this off unless you are deliberately A/B testing.
const DEFAULT_RUI_AUTO_CONNECT: bool = false;

/// Schema version of `config.json`, bumped when keys are renamed.
///
/// v1 (legacy config schema): `fivem_download_url`, `precache_url`,
///                           `rage_manifest_url`
/// v2:                     the fields below
/// v3:                     + `launch_target`
/// v4:                     + `rui_auto_connect`
pub const CONFIG_SCHEMA_VERSION: u32 = 4;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LauncherConfig {
    pub server_address: String,
    pub client_manifest_url: String,
    pub cache_info_url: String,
    pub patches_url: String,
    pub file_download_base_url: String,
    pub ragemp_download_url: String,
    pub rage_dir: String,
    pub gta5_path: String,
    /// Connect method used for direct join (`raw` | `hosts`).
    #[serde(default = "default_connect_mode")]
    pub connect_mode: String,
    /// Whitelisted domain used when `connect_mode == "hosts"`.
    #[serde(default)]
    pub hosts_domain: String,
    /// Which binary to run (`updater` | `ragemp_v`).
    #[serde(default = "default_launch_target")]
    pub launch_target: String,
    /// Whether to inject the auto-connect block into `rui\index.rui`.
    /// Off by default — the patch is opt-in after reports of it breaking the
    /// CEF launcher UI ("404 error: File not found").
    #[serde(default)]
    pub rui_auto_connect: bool,
    /// Binary patches for `ragemp_v.exe` (the game client itself).
    ///
    /// `ragemp_v.exe` rejects a raw IP written into `launch2.ip` — it only
    /// accepts `*.gta5rp.com` / `*.grandrp.com`. Patching the whitelist check
    /// here lets `connect_mode: "raw"` auto-connect by raw IP, which avoids the
    /// fake-domain hosts hack (that hack otherwise breaks the secure
    /// connection). The bytes are specific to the installed client build; leave
    /// empty to disable. Each entry is a `Patch` (`offset`/`original`/`replacement`).
    #[serde(default)]
    pub ragemp_v_patches: Vec<Patch>,
}

fn default_connect_mode() -> String {
    DEFAULT_CONNECT_MODE.to_string()
}

fn default_launch_target() -> String {
    DEFAULT_LAUNCH_TARGET.to_string()
}

fn config_path() -> std::path::PathBuf {
    launcher_data_dir().join("config.json")
}

/// First matching string value among `keys`, in priority order.
fn pick<'a>(json: &'a serde_json::Value, keys: &[&str]) -> Option<&'a str> {
    keys.iter().find_map(|k| json.get(*k).and_then(|v| v.as_str()))
}

/// Read the raw JSON object from disk, or an empty object.
fn read_raw() -> serde_json::Value {
    fs::read_to_string(config_path())
        .ok()
        .and_then(|s| serde_json::from_str::<serde_json::Value>(&s).ok())
        .filter(|v| v.is_object())
        .unwrap_or_else(|| serde_json::json!({}))
}

impl LauncherConfig {
    pub fn load() -> Self {
        let json = read_raw();

        if json.as_object().map(|o| o.is_empty()).unwrap_or(true) {
            log_warn!("config.json not found or invalid, creating with defaults");
            let config = Self::defaults();
            config.save();
            return config;
        }

        // ── Legacy key migration ──
        // Older builds stored legacy key names. Map them across so an existing
        // install keeps working instead of silently resetting to defaults.
        let mut migrated: Vec<(&str, &str)> = Vec::new();

        let client_manifest_url = match pick(&json, &["client_manifest_url"]) {
            Some(v) => v.to_string(),
            None => match pick(&json, &["rage_manifest_url"]) {
                Some(v) => {
                    migrated.push(("rage_manifest_url", "client_manifest_url"));
                    v.to_string()
                }
                None => DEFAULT_CLIENT_MANIFEST_URL.to_string(),
            },
        };

        let ragemp_download_url = match pick(&json, &["ragemp_download_url"]) {
            Some(v) => v.to_string(),
            None => match pick(&json, &["fivem_download_url"]) {
                Some(v) => {
                    migrated.push(("fivem_download_url", "ragemp_download_url"));
                    v.to_string()
                }
                None => DEFAULT_RAGEMP_DOWNLOAD_URL.to_string(),
            },
        };

        for (from, to) in &migrated {
            log_info!("config migration: '{}' -> '{}'", from, to);
        }

        let config = Self {
            server_address: pick(&json, &["server_address"])
                .unwrap_or(DEFAULT_SERVER_ADDRESS)
                .to_string(),
            client_manifest_url,
            cache_info_url: pick(&json, &["cache_info_url"])
                .unwrap_or(DEFAULT_CACHE_INFO_URL)
                .to_string(),
            patches_url: pick(&json, &["patches_url"])
                .unwrap_or(DEFAULT_PATCHES_URL)
                .to_string(),
            file_download_base_url: pick(&json, &["file_download_base_url"])
                .unwrap_or(DEFAULT_FILE_DOWNLOAD_BASE_URL)
                .to_string(),
            ragemp_download_url,
            rage_dir: pick(&json, &["rage_dir"]).unwrap_or("").to_string(),
            gta5_path: pick(&json, &["gta5_path"]).unwrap_or("").to_string(),
            connect_mode: pick(&json, &["connect_mode"])
                .unwrap_or(DEFAULT_CONNECT_MODE)
                .to_string(),
            hosts_domain: pick(&json, &["hosts_domain"])
                .unwrap_or(DEFAULT_HOSTS_DOMAIN)
                .to_string(),
            launch_target: pick(&json, &["launch_target"])
                .unwrap_or(DEFAULT_LAUNCH_TARGET)
                .to_string(),
            rui_auto_connect: json
                .get("rui_auto_connect")
                .and_then(|v| v.as_bool())
                .unwrap_or(DEFAULT_RUI_AUTO_CONNECT),
            ragemp_v_patches: json
                .get("ragemp_v_patches")
                .and_then(|v| serde_json::from_value(v.clone()).ok())
                .unwrap_or_default(),
        };

        // Persist the migration result so it only happens once.
        if !migrated.is_empty() {
            config.save();
        }

        config
    }

    fn defaults() -> Self {
        Self {
            server_address: DEFAULT_SERVER_ADDRESS.to_string(),
            client_manifest_url: DEFAULT_CLIENT_MANIFEST_URL.to_string(),
            cache_info_url: DEFAULT_CACHE_INFO_URL.to_string(),
            patches_url: DEFAULT_PATCHES_URL.to_string(),
            file_download_base_url: DEFAULT_FILE_DOWNLOAD_BASE_URL.to_string(),
            ragemp_download_url: DEFAULT_RAGEMP_DOWNLOAD_URL.to_string(),
            rage_dir: String::new(),
            gta5_path: String::new(),
            connect_mode: DEFAULT_CONNECT_MODE.to_string(),
            hosts_domain: DEFAULT_HOSTS_DOMAIN.to_string(),
            launch_target: DEFAULT_LAUNCH_TARGET.to_string(),
            rui_auto_connect: DEFAULT_RUI_AUTO_CONNECT,
            ragemp_v_patches: Vec::new(),
        }
    }

    /// Overwrite the connect-method fields and persist.
    pub fn set_connect_mode(&self, mode: &str, domain: &str) {
        let mut cfg = self.clone();
        cfg.connect_mode = mode.to_string();
        cfg.hosts_domain = domain.to_string();
        cfg.save();
    }

    /// Overwrite the launch target and persist.
    pub fn set_launch_target(&self, target: &str) {
        let mut cfg = self.clone();
        cfg.launch_target = target.to_string();
        cfg.save();
    }

    /// Toggle the rui auto-connect injection and persist.
    pub fn set_rui_auto_connect(&self, enabled: bool) {
        let mut cfg = self.clone();
        cfg.rui_auto_connect = enabled;
        cfg.save();
    }

    /// Merge-write our keys into `config.json`.
    ///
    /// Unknown keys written by other builds (or by the user) are **preserved**
    /// rather than dropped — a plain overwrite would silently destroy them.
    pub fn save(&self) {
        let mut json = read_raw();

        let obj = match json.as_object_mut() {
            Some(o) => o,
            None => {
                json = serde_json::json!({});
                json.as_object_mut().expect("just created an object")
            }
        };

        obj.insert(
            "config_version".into(),
            serde_json::json!(CONFIG_SCHEMA_VERSION),
        );
        obj.insert("server_address".into(), serde_json::json!(self.server_address));
        obj.insert(
            "client_manifest_url".into(),
            serde_json::json!(self.client_manifest_url),
        );
        obj.insert("cache_info_url".into(), serde_json::json!(self.cache_info_url));
        obj.insert("patches_url".into(), serde_json::json!(self.patches_url));
        obj.insert(
            "file_download_base_url".into(),
            serde_json::json!(self.file_download_base_url),
        );
        obj.insert(
            "ragemp_download_url".into(),
            serde_json::json!(self.ragemp_download_url),
        );
        obj.insert("rage_dir".into(), serde_json::json!(self.rage_dir));
        obj.insert("gta5_path".into(), serde_json::json!(self.gta5_path));
        obj.insert("connect_mode".into(), serde_json::json!(self.connect_mode));
        obj.insert("hosts_domain".into(), serde_json::json!(self.hosts_domain));
        obj.insert("launch_target".into(), serde_json::json!(self.launch_target));
        obj.insert(
            "rui_auto_connect".into(),
            serde_json::json!(self.rui_auto_connect),
        );
        obj.insert(
            "ragemp_v_patches".into(),
            serde_json::json!(self.ragemp_v_patches),
        );

        let path = config_path();
        let _ = fs::write(
            &path,
            serde_json::to_string_pretty(&json).unwrap_or_default(),
        );
        log_info!("Config saved to {:?}", path);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pick_prefers_the_first_key() {
        let json = serde_json::json!({ "a": "first", "b": "second" });
        assert_eq!(pick(&json, &["a", "b"]), Some("first"));
        assert_eq!(pick(&json, &["b", "a"]), Some("second"));
        assert_eq!(pick(&json, &["zz", "a"]), Some("first"));
        assert_eq!(pick(&json, &["nope"]), None);
    }

    #[test]
    fn pick_ignores_non_string_values() {
        let json = serde_json::json!({ "n": 5, "s": "ok" });
        assert_eq!(pick(&json, &["n"]), None);
        assert_eq!(pick(&json, &["n", "s"]), Some("ok"));
    }

    #[test]
    fn schema_version_is_four() {
        assert_eq!(CONFIG_SCHEMA_VERSION, 4);
    }
}
