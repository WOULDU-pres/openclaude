use std::collections::HashMap;
use std::sync::Arc;

use teloxide::prelude::*;
use tokio::sync::Mutex;

use crate::claude::CancelToken;

use super::storage::{BotSettings, ChatSession};

/// Shared state: per-chat sessions + bot settings
pub(crate) struct SharedData {
    pub(crate) sessions: HashMap<ChatId, ChatSession>,
    pub(crate) settings: BotSettings,
    /// Per-chat cancel tokens for stopping in-progress AI requests
    pub(crate) cancel_tokens: HashMap<ChatId, Arc<CancelToken>>,
    /// Message ID of the "Stopping..." message sent by /stop, so the polling loop can update it
    pub(crate) stop_message_ids: HashMap<ChatId, teloxide::types::MessageId>,
    /// Per-chat timestamp of the last Telegram API call (for rate limiting)
    pub(crate) api_timestamps: HashMap<ChatId, tokio::time::Instant>,
}

pub(crate) type SharedState = Arc<Mutex<SharedData>>;

/// Telegram message length limit
pub(crate) const TELEGRAM_MSG_LIMIT: usize = 4096;

/// Shared per-chat rate limiter using reservation pattern.
/// Acquires the lock briefly to calculate and reserve the next API call slot,
/// then releases the lock and sleeps until the reserved time.
/// This ensures that even concurrent tasks for the same chat maintain 3s gaps.
pub(crate) async fn shared_rate_limit_wait(state: &SharedState, chat_id: ChatId) {
    let min_gap = tokio::time::Duration::from_millis(3000);
    let sleep_until = {
        let mut data = state.lock().await;
        let last = data
            .api_timestamps
            .entry(chat_id)
            .or_insert_with(|| tokio::time::Instant::now() - tokio::time::Duration::from_secs(10));
        let earliest_next = *last + min_gap;
        let now = tokio::time::Instant::now();
        let target = if earliest_next > now {
            earliest_next
        } else {
            now
        };
        *last = target; // Reserve this slot
        target
    }; // Mutex released here
    tokio::time::sleep_until(sleep_until).await;
}
