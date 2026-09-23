//! Minimal Telegram long-poll remote (opt-in).
//!
//! The host implements [`CommandHandler`]: map text commands to actions
//! (status, pause, resume, quit). The kit only talks to Bot API.

use serde::Deserialize;
use std::time::Duration;
use thiserror::Error;

/// Telegram remote errors.
#[derive(Debug, Error)]
pub enum Error {
    /// HTTP / Bot API failure.
    #[error("Telegram request failed: {0}")]
    Http(String),
    /// Unexpected JSON.
    #[error("invalid Telegram JSON: {0}")]
    Json(String),
}

/// Bot credentials and optional chat allow-list.
#[derive(Debug, Clone)]
pub struct BotConfig {
    /// Bot token from @BotFather.
    pub token: String,
    /// If non-empty, only these chat ids may issue commands.
    pub allow_chats: Vec<i64>,
}

/// One inbound command from a user.
#[derive(Debug, Clone)]
pub struct Incoming {
    /// Telegram chat id.
    pub chat_id: i64,
    /// Message text (trimmed).
    pub text: String,
}

/// Host-provided reactions to bot commands.
pub trait CommandHandler {
    /// Handle one message; return an optional reply text.
    fn handle(&mut self, msg: &Incoming) -> Option<String>;
}

#[derive(Debug, Deserialize)]
struct ApiOk<T> {
    ok: bool,
    result: Option<T>,
    description: Option<String>,
}

#[derive(Debug, Deserialize)]
struct Update {
    update_id: i64,
    message: Option<Message>,
}

#[derive(Debug, Deserialize)]
struct Message {
    chat: Chat,
    text: Option<String>,
}

#[derive(Debug, Deserialize)]
struct Chat {
    id: i64,
}

/// Send a plain text message.
pub fn send_text(cfg: &BotConfig, chat_id: i64, text: &str) -> Result<(), Error> {
    let url = format!("https://api.telegram.org/bot{}/sendMessage", cfg.token);
    let body = serde_json::json!({
        "chat_id": chat_id,
        "text": text,
    });
    let resp: ApiOk<serde_json::Value> = ureq::post(&url)
        .set("Content-Type", "application/json")
        .send_json(body)
        .map_err(|e| Error::Http(e.to_string()))?
        .into_json()
        .map_err(|e| Error::Json(e.to_string()))?;
    if !resp.ok {
        return Err(Error::Http(
            resp.description.unwrap_or_else(|| "sendMessage failed".into()),
        ));
    }
    Ok(())
}

/// Poll `getUpdates` once (long poll up to `timeout` seconds) and dispatch.
///
/// Returns the next `offset` to pass on the following call (`update_id + 1`).
pub fn poll_once(
    cfg: &BotConfig,
    offset: i64,
    timeout_secs: u64,
    handler: &mut dyn CommandHandler,
) -> Result<i64, Error> {
    let url = format!("https://api.telegram.org/bot{}/getUpdates", cfg.token);
    let resp: ApiOk<Vec<Update>> = ureq::get(&url)
        .query("offset", &offset.to_string())
        .query("timeout", &timeout_secs.to_string())
        .timeout(Duration::from_secs(timeout_secs + 5))
        .call()
        .map_err(|e| Error::Http(e.to_string()))?
        .into_json()
        .map_err(|e| Error::Json(e.to_string()))?;

    if !resp.ok {
        return Err(Error::Http(
            resp.description.unwrap_or_else(|| "getUpdates failed".into()),
        ));
    }

    let mut next = offset;
    for u in resp.result.unwrap_or_default() {
        next = next.max(u.update_id + 1);
        let Some(msg) = u.message else { continue };
        let Some(text) = msg.text else { continue };
        let text = text.trim().to_string();
        if text.is_empty() {
            continue;
        }
        if !cfg.allow_chats.is_empty() && !cfg.allow_chats.contains(&msg.chat.id) {
            continue;
        }
        let incoming = Incoming {
            chat_id: msg.chat.id,
            text,
        };
        if let Some(reply) = handler.handle(&incoming) {
            let _ = send_text(cfg, incoming.chat_id, &reply);
        }
    }
    Ok(next)
}
