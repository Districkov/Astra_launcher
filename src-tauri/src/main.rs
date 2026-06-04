//! ASTRA Launcher — Backend (Rust / Tauri v2)
//! =========================================
//! Подход на основе bartosjiri/fivem-launcher:
//!   - Запуск через cmd.exe: FiveM.exe +connect IP:порт
//!   - Поиск FiveM.exe в подпапках (FiveM.app и др.)
//!   - Пользователь может указать путь вручную
//!   - Статус сервера через /info.json и /players.json
//!   - Встроенный webview для Telegram (add_child)

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::{command, Manager, Emitter, WebviewUrl};
use tauri::webview::WebviewBuilder;
use tauri::tray::{TrayIconBuilder, MouseButton, MouseButtonState, TrayIconEvent};
use tauri::menu::{MenuBuilder, MenuItemBuilder};
use discord_rich_presence::{DiscordIpc, DiscordIpcClient};
use std::sync::Mutex;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;
#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

/// Глобальное состояние Discord RPC
static DISCORD_RPC: once_cell::sync::Lazy<Mutex<Option<DiscordIpcClient>>> = once_cell::sync::Lazy::new(|| Mutex::new(None));

/// ─────────────────────────────────────────────
/// 🔧 КОНФИГУРАЦИЯ
/// ─────────────────────────────────────────────

/// IP:port вашего FiveM-сервера
const SERVER_ADDRESS: &str = "185.176.94.21:30120";

/// URL скачивания установщика FiveM (официальный с fivem.net)
const FIVEM_DOWNLOAD_URL: &str = "https://runtime.fivem.net/client/FiveM.exe";

/// ─────────────────────────────────────────────
/// 📁 ПУТИ
/// ─────────────────────────────────────────────

/// Папка данных лаунчера: %APPDATA%/astra-launcher/
fn launcher_data_dir() -> PathBuf {
    let appdata = dirs::data_dir().expect("Не удалось найти AppData");
    let dir = appdata.join("astra-launcher");
    fs::create_dir_all(&dir).ok();
    dir
}

/// Путь к файлу настроек (хранит путь к FiveM.exe)
fn settings_path() -> PathBuf {
    launcher_data_dir().join("settings.json")
}

/// Путь к скачанному установщику FiveM
fn fivem_installer_path() -> PathBuf {
    launcher_data_dir().join("FiveM_install.exe")
}

/// ─────────────────────────────────────────────
/// 🔍 ПОИСК FIVEM.EXE
/// ─────────────────────────────────────────────
/// FiveM устанавливается в %LocalAppData%\FiveM\
/// Структура может быть разной:
///   - FiveM\FiveM.exe
///   - FiveM\FiveM.app\FiveM.exe
///   - Program Files\FiveM\FiveM.exe
/// Пользователь также может указать путь вручную.

/// Ищет FiveM.exe во всех возможных местах.
/// Возвращает путь, если найден.
fn find_fivem_exe() -> Option<PathBuf> {
    // 1. Проверяем пользовательский путь из настроек
    if let Ok(data) = fs::read_to_string(settings_path()) {
        if let Ok(settings) = serde_json::from_str::<serde_json::Value>(&data) {
            if let Some(custom_path) = settings.get("fivem_path").and_then(|v| v.as_str()) {
                let path = PathBuf::from(custom_path);
                if path.exists() {
                    return Some(path);
                }
            }
        }
    }

    // 2. Стандартные пути установки (используем env::var для надёжности)
    let local_appdata_env = std::env::var("LOCALAPPDATA").ok();
    let home_env = std::env::var("USERPROFILE").ok();
    let local_appdata = dirs::data_local_dir().unwrap_or_default();
    let home = dirs::home_dir().unwrap_or_default();

    let mut search_paths: Vec<PathBuf> = vec![
        // %LocalAppData%\FiveM\FiveM.exe (стандарт)
        local_appdata.join("FiveM").join("FiveM.exe"),
        // %LocalAppData%\FiveM\FiveM.app\FiveM.exe (альтернатива)
        local_appdata.join("FiveM").join("FiveM.app").join("FiveM.exe"),
        // Program Files
        PathBuf::from(r"C:\Program Files\FiveM\FiveM.exe"),
        PathBuf::from(r"C:\Program Files (x86)\FiveM\FiveM.exe"),
        // Desktop
        home.join("Desktop").join("FiveM.exe"),
        // Дополнительные варианты
        local_appdata.join("FiveM Application").join("FiveM.exe"),
        PathBuf::from(r"D:\FiveM\FiveM.exe"),
    ];

    // Добавляем пути через env::var (более надёжно чем dirs crate)
    if let Some(la) = &local_appdata_env {
        search_paths.push(PathBuf::from(la).join("FiveM").join("FiveM.exe"));
        search_paths.push(PathBuf::from(la).join("FiveM").join("FiveM.app").join("FiveM.exe"));
    }
    if let Some(h) = &home_env {
        search_paths.push(PathBuf::from(h).join("Desktop").join("FiveM.exe"));
    }

    for path in &search_paths {
        if path.exists() {
            return Some(path.clone());
        }
    }

    // 3. Проверяем .lnk ярлыки на рабочем столе
    if let Some(h) = &home_env {
        let desktop_lnk = PathBuf::from(h).join("Desktop").join("FiveM.lnk");
        if desktop_lnk.exists() {
            if let Some(target) = resolve_lnk_target(&desktop_lnk) {
                if target.exists() {
                    return Some(target);
                }
            }
        }
    }
    let desktop_lnk = home.join("Desktop").join("FiveM.lnk");
    if desktop_lnk.exists() {
        if let Some(target) = resolve_lnk_target(&desktop_lnk) {
            if target.exists() {
                return Some(target);
            }
        }
    }

    // 4. Рекурсивный поиск в %LocalAppData%\FiveM\ (на случай нестандартной структуры)
    let fivem_dir = local_appdata.join("FiveM");
    if fivem_dir.exists() {
        if let Some(found) = search_exe_recursive(&fivem_dir, 3) {
            return Some(found);
        }
    }
    // Также через env::var
    if let Some(la) = &local_appdata_env {
        let fivem_dir_env = PathBuf::from(la).join("FiveM");
        if fivem_dir_env.exists() {
            if let Some(found) = search_exe_recursive(&fivem_dir_env, 3) {
                return Some(found);
            }
        }
    }

    None
}

/// Разрешает .lnk ярлык и возвращает путь к целевому файлу (через PowerShell)
#[cfg(target_os = "windows")]
fn resolve_lnk_target(lnk_path: &PathBuf) -> Option<PathBuf> {
    let output = Command::new("powershell")
        .args([
            "-NoProfile", "-NonInteractive", "-Command",
            &format!("(New-Object -ComObject WScript.Shell).CreateShortcut('{}').TargetPath",
                lnk_path.to_string_lossy().replace('\'', "''"))
        ])
        .creation_flags(0x00000008) // CREATE_NO_WINDOW
        .output()
        .ok()?;

    let target = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if target.is_empty() || !target.contains(':') {
        return None;
    }
    Some(PathBuf::from(target))
}

#[cfg(not(target_os = "windows"))]
fn resolve_lnk_target(_lnk_path: &PathBuf) -> Option<PathBuf> {
    None
}

/// Рекурсивно ищет FiveM.exe в папке (до max_depth уровней)
fn search_exe_recursive(dir: &PathBuf, max_depth: u32) -> Option<PathBuf> {
    if max_depth == 0 {
        return None;
    }

    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                if let Some(found) = search_exe_recursive(&path, max_depth - 1) {
                    return Some(found);
                }
            } else if path.file_name().and_then(|n| n.to_str()) == Some("FiveM.exe") {
                return Some(path);
            }
        }
    }

    None
}

/// ─────────────────────────────────────────────
/// 🎮 КОМАНДЫ: ПРОВЕРКА / ЗАПУСК FIVEM
/// ─────────────────────────────────────────────

/// Проверяет, установлен ли FiveM.
#[command]
fn check_fivem_installed() -> Result<bool, String> {
    Ok(find_fivem_exe().is_some())
}

/// Возвращает текущий путь к FiveM.exe (или пустую строку если не найден)
#[command]
fn get_fivem_path() -> Result<String, String> {
    match find_fivem_exe() {
        Some(path) => Ok(path.to_string_lossy().to_string()),
        None => Ok(String::new()),
    }
}

/// Сохраняет пользовательский путь к FiveM.exe
#[command]
fn set_fivem_path(path: String) -> Result<(), String> {
    let settings = serde_json::json!({
        "fivem_path": path
    });
    fs::write(settings_path(), serde_json::to_string_pretty(&settings).unwrap())
        .map_err(|e| format!("Ошибка сохранения: {}", e))
}

/// Автоматический поиск FiveM.exe — ищет и сохраняет путь, если найден
#[command]
fn auto_find_fivem() -> Result<Option<String>, String> {
    match find_fivem_exe() {
        Some(path) => {
            let path_str = path.to_string_lossy().to_string();
            let settings = serde_json::json!({
                "fivem_path": &path_str
            });
            let _ = fs::write(settings_path(), serde_json::to_string_pretty(&settings).unwrap());
            Ok(Some(path_str))
        }
        None => Ok(None),
    }
}

/// Запускает FiveM и подключается к серверу.
/// Запускает FiveM.exe через cmd /c start — FiveM увидит explorer как родителя
/// (cmd /c start открывает процесс через ShellExecuteEx, как если бы кликнули в проводнике)
#[command]
fn launch_game() -> Result<String, String> {
    let fivem_exe = find_fivem_exe().ok_or("FIVEM_NOT_FOUND")?;

    let exe_str = fivem_exe.to_string_lossy().to_string();

    // Валидация: путь должен заканчиваться на FiveM.exe
    if !exe_str.to_lowercase().ends_with("fivem.exe") {
        return Err("FIVEM_NOT_FOUND".to_string());
    }

    // FiveM проверяет, что запущен из оболочки (explorer.exe) или браузера
    // Ни spawn(), ни ShellExecuteW, ни cmd /c start НЕ работают — FiveM видит наш процесс
    // Единственный рабочий способ: explorer.exe + fivem:// протокол
    // explorer.exe — это и есть "shell", FiveM видит его как родителя и не крашится
    #[cfg(target_os = "windows")]
    {
        let url = format!("fivem://connect/{}", SERVER_ADDRESS);
        Command::new("explorer.exe")
            .arg(&url)
            .creation_flags(0x00000008) // CREATE_NO_WINDOW
            .spawn()
            .map_err(|e| format!("Не удалось запустить FiveM: {}", e))?;
    }

    #[cfg(not(target_os = "windows"))]
    {
        Command::new(&exe_str)
            .args(["+connect", SERVER_ADDRESS])
            .spawn()
            .map_err(|e| format!("Не удалось запустить FiveM: {}", e))?;
    }

    Ok(format!("Подключение к {}", SERVER_ADDRESS))
}

/// ─────────────────────────────────────────────
/// 📥 СКАЧИВАНИЕ FIVEM С ПРОГРЕССОМ
/// ─────────────────────────────────────────────

#[command]
async fn download_fivem(window: tauri::Window) -> Result<String, String> {
    let dest = fivem_installer_path();

    // Если установщик уже скачан — не скачиваем повторно
    if dest.exists() {
        if let Ok(meta) = fs::metadata(&dest) {
            if meta.len() > 1_000_000 {
                // Файл > 1 МБ — считаем что скачан полностью
                return Ok(dest.to_string_lossy().to_string());
            }
        }
        // Файл слишком мал (битый) — удаляем и скачиваем заново
        let _ = fs::remove_file(&dest);
    }

    let client = reqwest::Client::new();
    let response = client
        .get(FIVEM_DOWNLOAD_URL)
        .send()
        .await
        .map_err(|e| format!("Ошибка подключения: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("Сервер вернул статус {}", response.status()));
    }

    let total_size = response.content_length().unwrap_or(0);
    let mut downloaded: u64 = 0;

    let mut file =
        fs::File::create(&dest).map_err(|e| format!("Ошибка создания файла: {}", e))?;

    let mut stream = response.bytes_stream();
    use futures_util::StreamExt;

    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| format!("Ошибка чтения: {}", e))?;
        file.write_all(&chunk)
            .map_err(|e| format!("Ошибка записи: {}", e))?;
        downloaded += chunk.len() as u64;

        let percent = if total_size > 0 {
            (downloaded as f64 / total_size as f64 * 100.0) as u32
        } else {
            0
        };

        let payload = serde_json::json!({
            "downloaded": downloaded,
            "total": total_size,
            "percent": percent
        });
        let _ = window.emit("download-progress", &payload);
    }

    Ok(dest.to_string_lossy().to_string())
}

/// ─────────────────────────────────────────────
/// 📦 ЗАПУСК УСТАНОВЩИКА FIVEM
/// ─────────────────────────────────────────────

/// Запускает скачанный установщик FiveM БЕЗ ожидания.
#[command]
fn launch_fivem_installer() -> Result<String, String> {
    let installer = fivem_installer_path();

    if !installer.exists() {
        return Err("Установщик не найден. Скачайте FiveM сначала.".to_string());
    }

    std::process::Command::new(installer.as_os_str())
        .spawn()
        .map_err(|e| format!("Не удалось запустить установщик: {}", e))?;

    Ok("Установщик запущен".to_string())
}

/// ─────────────────────────────────────────────
/// 🌐 СТАТУС СЕРВЕРА
/// ─────────────────────────────────────────────

/// Получает статус сервера через /info.json и /players.json
#[command]
async fn get_server_status() -> Result<serde_json::Value, String> {
    let client = reqwest::Client::new();

    // Запрашиваем info.json
    let info = match client
        .get(format!("http://{}/info.json", SERVER_ADDRESS))
        .timeout(std::time::Duration::from_secs(5))
        .send()
        .await
    {
        Ok(resp) => resp.json::<serde_json::Value>().await.ok(),
        Err(_) => None,
    };

    // Запрашиваем players.json
    let players = match client
        .get(format!("http://{}/players.json", SERVER_ADDRESS))
        .timeout(std::time::Duration::from_secs(5))
        .send()
        .await
    {
        Ok(resp) => resp.json::<serde_json::Value>().await.ok(),
        Err(_) => None,
    };

    let online = info.is_some();
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

    Ok(serde_json::json!({
        "online": online,
        "players": players_count,
        "max_players": max_players,
        "info": info,
    }))
}

/// ─────────────────────────────────────────────
/// � ПРЕДЗАГРУЗКА КЕША СЕРВЕРА
/// ─────────────────────────────────────────────

/// URL для скачивания кеша сервера (ZIP-архив)
/// Замените на реальный URL вашего сервера
const PRECACHE_URL: &str = "http://185.176.94.21/precache.zip";

/// Путь к кешу FiveM: %LocalAppData%\FiveM\FiveM.app\data\server-cache-priv\
fn fivem_server_cache_dir() -> Option<PathBuf> {
    let local_appdata = std::env::var("LOCALAPPDATA").ok()?;
    Some(PathBuf::from(local_appdata)
        .join("FiveM")
        .join("FiveM.app")
        .join("data")
        .join("server-cache-priv"))
}

/// Проверяет, нужна ли предзагрузка (кеш пуст или почти пуст)
#[command]
fn check_precache_needed() -> Result<serde_json::Value, String> {
    let cache_dir = fivem_server_cache_dir()
        .ok_or("Не удалось найти папку кеша FiveM")?;

    if !cache_dir.exists() {
        return Ok(serde_json::json!({
            "needed": true,
            "reason": "Кеш FiveM не найден (первый запуск)"
        }));
    }

    // Считаем cache_* файлы
    let cache_files: Vec<_> = fs::read_dir(&cache_dir)
        .map_err(|e| format!("Ошибка чтения кеша: {}", e))?
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.file_name()
                .to_string_lossy()
                .starts_with("cache_")
        })
        .collect();

    let count = cache_files.len();
    let total_size: u64 = cache_files
        .iter()
        .filter_map(|f| f.metadata().ok())
        .map(|m| m.len())
        .sum();

    // Если кеш < 50 МБ или < 100 файлов — нужна предзагрузка
    let needed = count < 100 || total_size < 50_000_000;

    Ok(serde_json::json!({
        "needed": needed,
        "reason": if needed { "Кеш сервера пуст или неполный" } else { "Кеш сервера актуален" },
        "cache_files": count,
        "cache_size_mb": (total_size as f64 / 1_048_576.0).round()
    }))
}

/// Скачивает и распаковывает кеш сервера с прогрессом
#[command]
async fn precache_server_files(window: tauri::Window) -> Result<serde_json::Value, String> {
    let cache_dir = fivem_server_cache_dir()
        .ok_or("Не удалось найти папку кеша FiveM")?;

    // Создаём папку если нет
    fs::create_dir_all(&cache_dir)
        .map_err(|e| format!("Ошибка создания папки кеша: {}", e))?;

    // Путь для временного ZIP
    let zip_path = launcher_data_dir().join("precache.zip");

    // Скачиваем ZIP с прогрессом
    let client = reqwest::Client::new();
    let response = client
        .get(PRECACHE_URL)
        .timeout(std::time::Duration::from_secs(300)) // 5 мин на скачивание
        .send()
        .await
        .map_err(|e| format!("Ошибка подключения к серверу: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("Сервер вернул статус {} (возможно precache.zip ещё не создан)", response.status()));
    }

    let total_size = response.content_length().unwrap_or(0);
    let mut downloaded: u64 = 0;

    let mut file = fs::File::create(&zip_path)
        .map_err(|e| format!("Ошибка создания файла: {}", e))?;

    let mut stream = response.bytes_stream();
    use futures_util::StreamExt;

    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| format!("Ошибка чтения: {}", e))?;
        file.write_all(&chunk)
            .map_err(|e| format!("Ошибка записи: {}", e))?;
        downloaded += chunk.len() as u64;

        let percent = if total_size > 0 {
            (downloaded as f64 / total_size as f64 * 100.0) as u32
        } else {
            0
        };

        let payload = serde_json::json!({
            "phase": "download",
            "downloaded": downloaded,
            "total": total_size,
            "percent": percent
        });
        let _ = window.emit("precache-progress", &payload);
    }

    drop(file); // Закрываем файл перед распаковкой

    // Распаковываем ZIP
    let payload = serde_json::json!({
        "phase": "extract",
        "percent": 0
    });
    let _ = window.emit("precache-progress", &payload);

    let zip_file = fs::File::open(&zip_path)
        .map_err(|e| format!("Ошибка открытия ZIP: {}", e))?;

    let mut archive = zip::ZipArchive::new(zip_file)
        .map_err(|e| format!("Ошибка чтения ZIP: {}", e))?;

    let total_files = archive.len();
    let mut extracted = 0;

    for i in 0..total_files {
        let mut zip_file = archive.by_index(i)
            .map_err(|e| format!("Ошибка чтения файла из ZIP: {}", e))?;

        let outpath = match zip_file.enclosed_name() {
            Some(path) => cache_dir.join(path),
            None => continue,
        };

        // Создаём родительские папки
        if let Some(parent) = outpath.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Ошибка создания папки: {}", e))?;
        }

        // Пропускаем папки (созданы выше)
        if zip_file.is_dir() {
            continue;
        }

        // Копируем файл
        let mut outfile = fs::File::create(&outpath)
            .map_err(|e| format!("Ошибка создания файла: {}", e))?;
        std::io::copy(&mut zip_file, &mut outfile)
            .map_err(|e| format!("Ошибка записи файла: {}", e))?;

        extracted += 1;
        let percent = (extracted as f64 / total_files as f64 * 100.0) as u32;

        let payload = serde_json::json!({
            "phase": "extract",
            "percent": percent,
            "extracted": extracted,
            "total_files": total_files
        });
        let _ = window.emit("precache-progress", &payload);
    }

    // Удаляем временный ZIP
    let _ = fs::remove_file(&zip_path);

    Ok(serde_json::json!({
        "success": true,
        "extracted_files": extracted,
        "cache_dir": cache_dir.to_string_lossy().to_string()
    }))
}

/// ─────────────────────────────────────────────
/// �🖥️ УПРАВЛЕНИЕ ОКНАМИ
/// ─────────────────────────────────────────────

#[command]
fn show_main_window(window: tauri::Window) -> Result<(), String> {
    let app = window.app_handle();

    let main_webview_window = app
        .get_webview_window("main")
        .ok_or("Главное окно не найдено")?;

    main_webview_window.show().map_err(|e| format!("Ошибка показа окна: {}", e))?;
    main_webview_window.set_focus().map_err(|e| format!("Ошибка фокуса: {}", e))?;

    window.close().map_err(|e| format!("Ошибка закрытия splash: {}", e))?;

    Ok(())
}

/// Создаёт встроенный webview внутри главного окна для отображения Telegram
#[command]
async fn create_embedded_webview(url: String, window: tauri::Window) -> Result<(), String> {
    let app = window.app_handle();

    // Проверяем, не существует ли уже webview с этой меткой
    if app.get_webview("embedded-webview").is_some() {
        // Уже есть — просто показываем
        return Ok(());
    }

    let parsed_url: url::Url = url
        .parse()
        .map_err(|e: url::ParseError| format!("Некорректный URL: {}", e))?;

    // Получаем нативное окно (не WebviewWindow) — add_child доступен на Window
    let main_window = app
        .get_window("main")
        .ok_or("Главное окно не найдено")?;

    // Позиция: справа от сайдбара (157px), ниже кнопок управления окном (32px сверху)
    let position = tauri::LogicalPosition::new(157.0, 32.0);
    let size = tauri::LogicalSize::new(803.0, 568.0);

    let webview_builder = WebviewBuilder::new("embedded-webview", WebviewUrl::External(parsed_url))
        .auto_resize();

    main_window
        .add_child(webview_builder, position, size)
        .map_err(|e| format!("Ошибка создания webview: {}", e))?;

    Ok(())
}

/// Закрывает встроенный webview
#[command]
fn close_embedded_webview(window: tauri::Window) -> Result<(), String> {
    let app = window.app_handle();

    if let Some(webview) = app.get_webview("embedded-webview") {
        webview.close().map_err(|e| format!("Ошибка закрытия webview: {}", e))?;
    }

    Ok(())
}

/// ─────────────────────────────────────────────
/// 🔄 АВТООБНОВЛЕНИЕ
/// ─────────────────────────────────────────────

/// Текущая версия лаунчера (должна совпадать с Cargo.toml)
const LAUNCHER_VERSION: &str = "1.3.0";

/// URL для проверки обновлений (GitHub releases API)
const UPDATE_CHECK_URL: &str = "https://api.github.com/repos/Districkov/Astra_launcher/releases/latest";

/// Проверяет наличие обновлений через GitHub Releases API.
/// Возвращает JSON с информацией об обновлении, если доступно.
#[command]
async fn check_for_updates() -> Result<serde_json::Value, String> {
    let client = reqwest::Client::new();

    let response = client
        .get(UPDATE_CHECK_URL)
        .header("User-Agent", "ASTRA-Launcher")
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await
        .map_err(|e| format!("Ошибка проверки обновлений: {}", e))?;

    if !response.status().is_success() {
        // Не критичная ошибка — просто возвращаем "нет обновлений"
        return Ok(serde_json::json!({
            "update_available": false,
            "current_version": LAUNCHER_VERSION,
            "error": format!("HTTP {}", response.status())
        }));
    }

    let release: serde_json::Value = response
        .json()
        .await
        .map_err(|e| format!("Ошибка парсинга ответа: {}", e))?;

    let latest_tag = release
        .get("tag_name")
        .and_then(|t| t.as_str())
        .unwrap_or(LAUNCHER_VERSION)
        .trim_start_matches('v');

    // Ищем .exe установщик (NSIS) среди assets
    // Формат имени: ASTRA.Launcher_X.Y.Z_x64-setup.exe или ASTRA Launcher_X.Y.Z_x64-setup.exe
    let download_url = release
        .get("assets")
        .and_then(|a| a.as_array())
        .and_then(|assets| {
            assets.iter().find(|asset| {
                asset.get("name")
                    .and_then(|n| n.as_str())
                    .map(|name| {
                        name.ends_with("-setup.exe") || name.ends_with("_setup.exe") || (name.contains("setup") && name.ends_with(".exe"))
                    })
                    .unwrap_or(false)
            })
        })
        .and_then(|asset| asset.get("browser_download_url"))
        .and_then(|u| u.as_str())
        .unwrap_or("")
        .to_string();

    let release_notes = release
        .get("body")
        .and_then(|b| b.as_str())
        .unwrap_or("")
        .to_string();

    // Сравнение версий: парсим major.minor.patch
    let update_available = {
        let cur: Vec<u32> = LAUNCHER_VERSION.split('.').filter_map(|s| s.parse().ok()).collect();
        let lat: Vec<u32> = latest_tag.split('.').filter_map(|s| s.parse().ok()).collect();
        // Сравниваем численно: latest > current → обновление доступно
        let lat_major = lat.get(0).copied().unwrap_or(0);
        let lat_minor = lat.get(1).copied().unwrap_or(0);
        let lat_patch = lat.get(2).copied().unwrap_or(0);
        let cur_major = cur.get(0).copied().unwrap_or(0);
        let cur_minor = cur.get(1).copied().unwrap_or(0);
        let cur_patch = cur.get(2).copied().unwrap_or(0);
        (lat_major, lat_minor, lat_patch) > (cur_major, cur_minor, cur_patch)
    };

    Ok(serde_json::json!({
        "update_available": update_available,
        "current_version": LAUNCHER_VERSION,
        "latest_version": latest_tag,
        "download_url": download_url,
        "release_notes": release_notes
    }))
}

/// Скачивает обновление (NSIS .exe) с прогрессом
#[command]
async fn download_update(url: String, window: tauri::Window) -> Result<String, String> {
    if url.is_empty() {
        return Err("URL скачивания пуст".to_string());
    }

    let client = reqwest::Client::new();
    let response = client
        .get(&url)
        .header("User-Agent", "ASTRA-Launcher-Updater")
        .timeout(std::time::Duration::from_secs(300)) // 5 мин
        .send()
        .await
        .map_err(|e| format!("Ошибка подключения: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("Сервер вернул статус {}", response.status()));
    }

    let total_size = response.content_length().unwrap_or(0);
    let mut downloaded: u64 = 0;

    // Путь: %APPDATA%/astra-launcher/update/setup.exe
    let update_dir = launcher_data_dir().join("update");
    fs::create_dir_all(&update_dir)
        .map_err(|e| format!("Ошибка создания папки: {}", e))?;

    let setup_path = update_dir.join("ASTRA_Launcher_update_setup.exe");
    let mut file = fs::File::create(&setup_path)
        .map_err(|e| format!("Ошибка создания файла: {}", e))?;

    let mut stream = response.bytes_stream();
    use futures_util::StreamExt;

    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| format!("Ошибка чтения: {}", e))?;
        file.write_all(&chunk)
            .map_err(|e| format!("Ошибка записи: {}", e))?;
        downloaded += chunk.len() as u64;

        let percent = if total_size > 0 {
            (downloaded as f64 / total_size as f64 * 100.0) as u32
        } else {
            // Если размер неизвестен — считаем по мегабайтам (примерно 5 МБ)
            std::cmp::min((downloaded as f64 / 5_000_000.0 * 100.0) as u32, 99)
        };

        let payload = serde_json::json!({
            "downloaded": downloaded,
            "total": total_size,
            "percent": percent
        });
        let _ = window.emit("update-download-progress", &payload);
    }

    drop(file);

    Ok(setup_path.to_string_lossy().to_string())
}

/// Запускает скачанный установщик и закрывает лаунчер
#[command]
fn install_update(setup_path: String) -> Result<(), String> {
    let path = PathBuf::from(&setup_path);
    if !path.exists() {
        return Err("Файл установщика не найден".to_string());
    }

    // Запускаем установщик (NSIS с ключом /S — тихая установка, или без — с GUI)
    Command::new(&setup_path)
        .creation_flags(0x00000008) // DETACHED_PROCESS
        .spawn()
        .map_err(|e| format!("Ошибка запуска установщика: {}", e))?;

    // Даём установщику секунду на запуск, затем закрываем лаунчер
    std::thread::sleep(std::time::Duration::from_millis(500));

    // Закрываем лаунчер
    std::process::exit(0);
}

/// ─────────────────────────────────────────────
/// 👤 ИМЯ ПОЛЬЗОВАТЕЛЯ
/// ─────────────────────────────────────────────

/// Получает имя пользователя из конфига FiveM.
/// FiveM хранит имя в файле %AppData%\FiveM\FiveM.app\citizen\common\data\ или
/// в реестре / кэше. Пробуем несколько вариантов.
#[command]
fn get_username() -> Result<String, String> {
    // 1. Проверяем кэш лаунчера (если пользователь уже заходил)
    let cached_name_path = launcher_data_dir().join("cached_username.txt");
    if let Ok(name) = fs::read_to_string(&cached_name_path) {
        let trimmed = name.trim().to_string();
        if !trimmed.is_empty() {
            return Ok(trimmed);
        }
    }

    // 2. Пытаемся прочитать из реестра Windows (FiveM сохраняет имя там)
    let username = get_fivem_username_from_registry()
        .or_else(|| get_fivem_username_from_config())
        .unwrap_or_else(|| "Player".to_string());

    // Кэшируем найденное имя
    let _ = fs::write(&cached_name_path, &username);

    Ok(username)
}

/// Читает имя из реестра Windows (FiveM может хранить его там)
fn get_fivem_username_from_registry() -> Option<String> {
    use std::process::Command as Cmd;

    let output = Cmd::new("reg")
        .args(["query", r"HKCU\Software\FiveM", "/v", "name"])
        .output()
        .ok()?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    // Парсим вывод реестра: ищем значение после "REG_SZ"
    for line in stdout.lines() {
        if line.contains("REG_SZ") {
            let parts: Vec<&str> = line.split("REG_SZ").collect();
            if parts.len() > 1 {
                let name = parts[1].trim().to_string();
                if !name.is_empty() {
                    return Some(name);
                }
            }
        }
    }
    None
}

/// Читает имя из конфига FiveM
fn get_fivem_username_from_config() -> Option<String> {
    let local_appdata = dirs::data_local_dir()?;

    // Вариант 1: FiveM.app/citizen/common.cfg
    let cfg_path = local_appdata.join("FiveM").join("FiveM.app").join("citizen").join("common.cfg");
    if let Ok(content) = fs::read_to_string(&cfg_path) {
        for line in content.lines() {
            if line.starts_with("set name ") {
                let name = line.trim_start_matches("set name ").trim().trim_matches('"').to_string();
                if !name.is_empty() {
                    return Some(name);
                }
            }
        }
    }

    // Вариант 2: Читаем из cache.xml
    let cache_path = local_appdata.join("FiveM").join("FiveM.app").join("cache.xml");
    if let Ok(content) = fs::read_to_string(&cache_path) {
        // Простой XML парсинг — ищем <Name>...</Name>
        if let Some(start) = content.find("<Name>") {
            if let Some(end) = content.find("</Name>") {
                let name = content[start + 6..end].trim().to_string();
                if !name.is_empty() {
                    return Some(name);
                }
            }
        }
    }

    None
}

/// Сохраняет имя пользователя в кэш лаунчера
#[command]
fn save_username(name: String) -> Result<(), String> {
    let cached_name_path = launcher_data_dir().join("cached_username.txt");
    fs::write(&cached_name_path, &name)
        .map_err(|e| format!("Ошибка сохранения имени: {}", e))
}

/// ─────────────────────────────────────────────
/// 📶 ПИНГ СЕРВЕРА (#2)
/// ─────────────────────────────────────────────

/// Измеряет пинг до сервера (TCP connect time в ms)
#[command]
async fn get_server_ping() -> Result<serde_json::Value, String> {
    use std::net::TcpStream;
    use std::time::Instant;

    let addr = SERVER_ADDRESS;
    let start = Instant::now();

    match TcpStream::connect_timeout(
        &addr.parse().map_err(|e: std::net::AddrParseError| format!("Некорректный адрес: {}", e))?,
        std::time::Duration::from_secs(5),
    ) {
        Ok(_) => {
            let ping_ms = start.elapsed().as_millis() as u64;
            Ok(serde_json::json!({ "ping": ping_ms }))
        }
        Err(e) => Ok(serde_json::json!({ "ping": 0, "error": format!("{}", e) })),
    }
}

/// ─────────────────────────────────────────────
/// 📸 СКРИНШОТ-МЕНЕДЖЕР (#15)
/// ─────────────────────────────────────────────

/// ─────────────────────────────────────────────
/// 🖼️ ГАЛЕРЕЯ (catbox.moe API — загрузка)
/// ─────────────────────────────────────────────

/// Загружает скриншот на catbox.moe
#[command]
async fn upload_to_gallery(image_path: String, author: String) -> Result<serde_json::Value, String> {
    // Читаем файл
    let image_data = fs::read(&image_path)
        .map_err(|e| format!("Не удалось прочитать файл: {}", e))?;

    let file_name = PathBuf::from(&image_path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("screenshot.png")
        .to_string();

    // Определяем MIME-тип
    let mime = if file_name.ends_with(".jpg") || file_name.ends_with(".jpeg") {
        "image/jpeg"
    } else if file_name.ends_with(".webp") {
        "image/webp"
    } else if file_name.ends_with(".gif") {
        "image/gif"
    } else {
        "image/png"
    };

    // Используем reqwest::multipart для надёжной загрузки
    let part = reqwest::multipart::Part::bytes(image_data)
        .file_name(file_name.clone())
        .mime_str(mime)
        .unwrap_or_else(|_| reqwest::multipart::Part::bytes(Vec::new()).file_name("file"));

    let form = reqwest::multipart::Form::new()
        .text("reqtype", "fileupload")
        .part("fileToUpload", part);

    // Загружаем на catbox.moe
    let client = reqwest::Client::new();
    let response = client
        .post("https://catbox.moe/user/api.php")
        .multipart(form)
        .timeout(std::time::Duration::from_secs(60))
        .send()
        .await
        .map_err(|e| format!("Ошибка загрузки: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("Сервер вернул статус {}", response.status()));
    }

    let url = response.text().await.map_err(|e| format!("Ошибка чтения ответа: {}", e))?;
    let url = url.trim().to_string();

    if !url.starts_with("https://") {
        return Err(format!("Некорректный ответ: {}", url));
    }

    // Генерируем ID из URL
    let img_id = url.split('/').last().unwrap_or("unknown").to_string();

    Ok(serde_json::json!({
        "id": img_id,
        "url": url,
        "thumb": url,
        "author": author,
        "timestamp": chrono::Utc::now().timestamp(),
    }))
}

/// ─────────────────────────────────────────────
/// 🎬 ОНБОРДИНГ (первый запуск)
/// ─────────────────────────────────────────────

/// Путь к файлу флага онбординга
fn onboarding_flag_path() -> PathBuf {
    launcher_data_dir().join(".onboarding_complete")
}

/// Проверяет, пройден ли онбординг (первый запуск)
#[command]
fn is_onboarding_complete() -> Result<bool, String> {
    Ok(onboarding_flag_path().exists())
}

/// Отмечает онбординг как пройденный
#[command]
fn complete_onboarding() -> Result<(), String> {
    fs::write(onboarding_flag_path(), chrono::Utc::now().to_rfc3339())
        .map_err(|e| format!("Ошибка записи флага онбординга: {}", e))
}

/// ─────────────────────────────────────────────
/// 🖥️ СВЕРНУТЬ В ТРЕЙ (#9)
/// ─────────────────────────────────────────────

/// Сворачивает главное окно (вместо закрытия)
#[command]
fn minimize_to_tray(window: tauri::Window) -> Result<(), String> {
    window.minimize().map_err(|e| format!("Ошибка сворачивания: {}", e))
}

/// Сворачивает окно (кнопка —)
#[command]
fn window_minimize(window: tauri::Window) -> Result<(), String> {
    window.minimize().map_err(|e| format!("Ошибка сворачивания: {}", e))
}

/// Закрывает окно (кнопка ✕)
#[command]
fn window_close(window: tauri::Window) -> Result<(), String> {
    window.close().map_err(|e| format!("Ошибка закрытия: {}", e))
}

/// ─────────────────────────────────────────────
/// 🎮 DISCORD RICH PRESENCE (#13)
/// ─────────────────────────────────────────────

/// Discord Application ID (замените на свой при необходимости)
const DISCORD_APP_ID: &str = "1511825412911792241";

/// Подключает Discord RPC и устанавливает статус
#[command]
fn set_discord_rpc(state: String, details: String, large_text: String) -> Result<bool, String> {
    let mut rpc = DISCORD_RPC.lock().map_err(|e| format!("Ошибка блокировки: {}", e))?;

    if rpc.is_none() {
        // Подключаемся к Discord IPC
        let mut client = DiscordIpcClient::new(DISCORD_APP_ID)
            .map_err(|e| format!("Ошибка создания Discord IPC: {}", e))?;

        match client.connect() {
            Ok(_) => {
                *rpc = Some(client);
            }
            Err(e) => {
                let _ = e;
                return Ok(false);
            }
        }
    }

    if let Some(ref mut client) = *rpc {
        let activity = discord_rich_presence::activity::Activity::new()
            .state(&state)
            .details(&details)
            .assets(
                discord_rich_presence::activity::Assets::new()
                    .large_image("astra-logo")
                    .large_text(&large_text)
            )
            .timestamps(
                discord_rich_presence::activity::Timestamps::new()
                    .start(chrono::Utc::now().timestamp())
            );

        client.set_activity(activity)
            .map_err(|e| format!("Ошибка установки активности: {}", e))?;
        Ok(true)
    } else {
        Ok(false)
    }
}

/// Отключает Discord RPC
#[command]
fn clear_discord_rpc() -> Result<(), String> {
    let mut rpc = DISCORD_RPC.lock().map_err(|e| format!("Ошибка блокировки: {}", e))?;
    if let Some(ref mut client) = *rpc {
        client.close().ok();
    }
    *rpc = None;
    Ok(())
}

/// ─────────────────────────────────────────────
/// 🏁 ТОЧКА ВХОДА
/// ─────────────────────────────────────────────

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_notification::init())
        .setup(|app| {
            // #9 — Системный трей
            let show_item = MenuItemBuilder::with_id("show", "Показать лаунчер").build(app)?;
            let quit_item = MenuItemBuilder::with_id("quit", "Выход").build(app)?;
            let menu = MenuBuilder::new(app)
                .item(&show_item)
                .separator()
                .item(&quit_item)
                .build()?;

            let _tray = TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .tooltip("ASTRA Launcher")
                .menu(&menu)
                .on_menu_event(|app, event| {
                    match event.id.as_ref() {
                        "show" => {
                            if let Some(w) = app.get_webview_window("main") {
                                let _ = w.show();
                                let _ = w.set_focus();
                            }
                        }
                        "quit" => {
                            app.exit(0);
                        }
                        _ => {}
                    }
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click { button: MouseButton::Left, button_state: MouseButtonState::Up, .. } = event {
                        let app = tray.app_handle();
                        if let Some(w) = app.get_webview_window("main") {
                            let _ = w.show();
                            let _ = w.set_focus();
                        }
                    }
                })
                .build(app)?;

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Splash / установка
            check_fivem_installed,
            get_fivem_path,
            set_fivem_path,
            auto_find_fivem,
            download_fivem,
            launch_fivem_installer,
            show_main_window,
            // Лаунчер
            launch_game,
            get_server_status,
            // Встроенный браузер
            create_embedded_webview,
            close_embedded_webview,
            // Обновления
            check_for_updates,
            download_update,
            install_update,
            // Имя пользователя
            get_username,
            save_username,
            // Пинг сервера
            get_server_ping,
            // Предзагрузка кеша сервера
            check_precache_needed,
            precache_server_files,
            // Галерея (загрузка на catbox.moe)
            upload_to_gallery,
            // Трей
            minimize_to_tray,
            // Управление окном
            window_minimize,
            window_close,
            // Discord Rich Presence
            set_discord_rpc,
            clear_discord_rpc,
            // Онбординг (первый запуск)
            is_onboarding_complete,
            complete_onboarding,
        ])
        .run(tauri::generate_context!())
        .expect("Ошибка при запуске Tauri");
}
