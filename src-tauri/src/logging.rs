use std::fs;
use std::io::Write;

use crate::paths::launcher_data_dir;

pub fn log_to_file(level: &str, msg: &str) {
    let log_path = launcher_data_dir().join("launcher.log");
    let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");
    let line = format!("[{} {}] {}\n", timestamp, level, msg);
    let _ = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)
        .and_then(|mut f| f.write_all(line.as_bytes()));
}

macro_rules! log_info {
    ($($arg:tt)*) => {
        crate::logging::log_to_file("INFO", &format!($($arg)*))
    }
}

macro_rules! log_warn {
    ($($arg:tt)*) => {
        crate::logging::log_to_file("WARN", &format!($($arg)*))
    }
}

macro_rules! log_error {
    ($($arg:tt)*) => {
        crate::logging::log_to_file("ERROR", &format!($($arg)*))
    }
}

pub(crate) use log_error;
pub(crate) use log_info;
pub(crate) use log_warn;
