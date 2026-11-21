use tauri::Manager;

use crate::logging::log_info;

pub fn show_main_window(window: &tauri::Window) -> Result<(), String> {
    let app = window.app_handle();

    let main_webview_window = app
        .get_webview_window("main")
        .ok_or("Main window not found")?;

    main_webview_window
        .show()
        .map_err(|e| format!("Failed to show window: {}", e))?;
    main_webview_window
        .set_focus()
        .map_err(|e| format!("Failed to focus window: {}", e))?;

    window
        .close()
        .map_err(|e| format!("Failed to close splash: {}", e))?;

    Ok(())
}

pub async fn create_embedded_webview(url: String, window: &tauri::Window) -> Result<(), String> {
    log_info!("create_embedded_webview: URL={}", url);
    let app = window.app_handle();

    if app.get_webview("embedded-webview").is_some() {
        return Ok(());
    }

    let parsed_url: url::Url = url
        .parse()
        .map_err(|e: url::ParseError| format!("Invalid URL: {}", e))?;

    // Validate: only allow t.me and telegram.org
    let host = parsed_url.host_str().unwrap_or("");
    let allowed_hosts = [
        "t.me",
        "www.t.me",
        "telegram.org",
        "web.telegram.org",
        "oauth.telegram.org",
    ];
    if !allowed_hosts
        .iter()
        .any(|&h| host == h || host.ends_with(&format!(".{}", h)))
    {
        return Err(format!(
            "URL not allowed: {}. Only t.me and telegram.org allowed",
            host
        ));
    }

    let main_window = app.get_window("main").ok_or("Main window not found")?;

    let position = tauri::LogicalPosition::new(157.0, 32.0);
    let size = tauri::LogicalSize::new(803.0, 568.0);

    let webview_builder = tauri::webview::WebviewBuilder::new(
        "embedded-webview",
        tauri::WebviewUrl::External(parsed_url),
    )
    .auto_resize();

    main_window
        .add_child(webview_builder, position, size)
        .map_err(|e| format!("Failed to create webview: {}", e))?;

    Ok(())
}

pub fn close_embedded_webview(window: &tauri::Window) -> Result<(), String> {
    let app = window.app_handle();

    if let Some(webview) = app.get_webview("embedded-webview") {
        webview
            .close()
            .map_err(|e| format!("Failed to close webview: {}", e))?;
    }

    Ok(())
}

pub fn window_minimize(window: &tauri::Window) -> Result<(), String> {
    window
        .minimize()
        .map_err(|e| format!("Failed to minimize: {}", e))
}

pub fn window_close(window: &tauri::Window) -> Result<(), String> {
    window
        .close()
        .map_err(|e| format!("Failed to close: {}", e))
}
