use std::fs;
use std::path::PathBuf;

pub fn launcher_data_dir() -> PathBuf {
    let appdata = dirs::data_dir().expect("Не удалось найти AppData");
    let dir = appdata.join("astra-launcher");
    fs::create_dir_all(&dir).ok();
    dir
}

pub fn settings_path() -> PathBuf {
    launcher_data_dir().join("settings.json")
}

pub fn onboarding_flag_path() -> PathBuf {
    launcher_data_dir().join(".onboarding_complete")
}

pub fn cached_username_path() -> PathBuf {
    launcher_data_dir().join("cached_username.txt")
}
