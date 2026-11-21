use std::sync::Mutex;

use discord_rich_presence::{DiscordIpc, DiscordIpcClient};

use crate::logging::log_error;

const DISCORD_APP_ID: &str = "1511825412911792241";

static DISCORD_RPC: once_cell::sync::Lazy<Mutex<Option<DiscordIpcClient>>> =
    once_cell::sync::Lazy::new(|| Mutex::new(None));

pub fn set_discord_rpc(state: &str, details: &str, large_text: &str) -> Result<bool, String> {
    let mut rpc = DISCORD_RPC
        .lock()
        .map_err(|e| format!("Lock error: {}", e))?;

    if rpc.is_none() {
        let mut client = DiscordIpcClient::new(DISCORD_APP_ID)
            .map_err(|e| format!("Failed to create Discord IPC: {}", e))?;

        match client.connect() {
            Ok(_) => {
                *rpc = Some(client);
            }
            Err(e) => {
                log_error!("Discord IPC connect failed: {}", e);
                return Ok(false);
            }
        }
    }

    if let Some(ref mut client) = *rpc {
        let activity = discord_rich_presence::activity::Activity::new()
            .state(state)
            .details(details)
            .assets(
                discord_rich_presence::activity::Assets::new()
                    .large_image("astra-logo")
                    .large_text(large_text),
            )
            .timestamps(
                discord_rich_presence::activity::Timestamps::new()
                    .start(chrono::Utc::now().timestamp()),
            );

        client
            .set_activity(activity)
            .map_err(|e| format!("Failed to set activity: {}", e))?;
        Ok(true)
    } else {
        Ok(false)
    }
}

