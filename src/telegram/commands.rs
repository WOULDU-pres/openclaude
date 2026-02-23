use std::path::Path;
use std::sync::atomic::Ordering;

use teloxide::prelude::*;
use teloxide::types::ParseMode;

use crate::auth::is_path_within_sandbox;
use crate::session::HistoryType;

use super::bot::{shared_rate_limit_wait, SharedState};
use super::storage::{load_existing_session, save_bot_settings, ChatSession};
use super::streaming::send_long_message;

/// Handle /help command
pub(crate) async fn handle_help_command(
    bot: &Bot,
    chat_id: ChatId,
    state: &SharedState,
) -> ResponseResult<()> {
    let help = format!(
        "\
<b>{} Telegram Bot</b>
Manage server files &amp; chat with Claude Code AI.

<b>Session</b>
<code>/start &lt;path&gt;</code> — Start session at directory
<code>/start</code> — Start in default startup project directory
<code>/pwd</code> — Show current working directory
<code>/cd &lt;path&gt;</code> — Change working directory
<code>/clear</code> — Clear AI conversation history
<code>/stop</code> — Stop current AI request

<b>File Transfer</b>
<code>/down &lt;file&gt;</code> — Download file from server
Send a file/photo — Upload to session directory

<b>Shell</b>
<code>!&lt;command&gt;</code> — Run shell command directly
  e.g. <code>!ls -la</code>, <code>!git status</code>

<b>AI Chat</b>
Any other message is sent to Claude Code AI.
AI can read, edit, and run commands in your session.

<b>Tool Management</b>
<code>/availabletools</code> — List all available tools
<code>/allowedtools</code> — Show currently allowed tools
<code>/allowed +name</code> — Add tool (e.g. <code>/allowed +Bash</code>)
<code>/allowed -name</code> — Remove tool

<b>Group Chat</b>
<code>;</code><i>message</i> — Send message to AI
<code>;</code><i>caption</i> — Upload file with AI prompt
<code>/public on</code> — Allow all members to use bot
<code>/public off</code> — Owner only (default)

<code>/help</code> — Show this help",
        env!("CARGO_BIN_NAME")
    );

    shared_rate_limit_wait(state, chat_id).await;
    bot.send_message(chat_id, help)
        .parse_mode(ParseMode::Html)
        .await?;

    Ok(())
}

/// Handle /start <path> command
pub(crate) async fn handle_start_command(
    bot: &Bot,
    chat_id: ChatId,
    text: &str,
    state: &SharedState,
    token: &str,
    default_project_dir: &str,
) -> ResponseResult<()> {
    // Extract path from "/start <path>"
    let path_str = text.strip_prefix("/start").unwrap_or("").trim();

    let canonical_path = if path_str.is_empty() {
        // Bind to startup project directory by default.
        let path = Path::new(default_project_dir);
        if !path.exists() || !path.is_dir() {
            shared_rate_limit_wait(state, chat_id).await;
            bot.send_message(
                chat_id,
                format!(
                    "Error: default project dir is invalid: {}",
                    default_project_dir
                ),
            )
            .await?;
            return Ok(());
        }
        path.canonicalize()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|_| default_project_dir.to_string())
    } else {
        // Expand ~ to home directory
        let expanded = if path_str.starts_with("~/") || path_str == "~" {
            if let Some(home) = dirs::home_dir() {
                home.join(path_str.strip_prefix("~/").unwrap_or(""))
                    .display()
                    .to_string()
            } else {
                path_str.to_string()
            }
        } else {
            path_str.to_string()
        };
        // Validate path exists
        let path = Path::new(&expanded);
        if !path.exists() || !path.is_dir() {
            shared_rate_limit_wait(state, chat_id).await;
            bot.send_message(
                chat_id,
                format!("Error: '{}' is not a valid directory.", expanded),
            )
            .await?;
            return Ok(());
        }
        path.canonicalize()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|_| expanded)
    };

    // Sandbox check: target must be within the home directory
    let sandbox_root = dirs::home_dir().unwrap_or_else(|| Path::new("/").to_path_buf());
    if !is_path_within_sandbox(Path::new(&canonical_path), &sandbox_root) {
        shared_rate_limit_wait(state, chat_id).await;
        bot.send_message(
            chat_id,
            format!(
                "Access denied: '{}' is outside the allowed path sandbox.",
                canonical_path
            ),
        )
        .await?;
        return Ok(());
    }

    // Try to load existing session for this path
    let existing = load_existing_session(&canonical_path);

    let mut response_lines = Vec::new();

    {
        let mut data = state.lock().await;
        let session = data.sessions.entry(chat_id).or_insert_with(|| ChatSession {
            session_id: None,
            current_path: None,
            history: Vec::new(),
            pending_uploads: Vec::new(),
            cleared: false,
        });

        if let Some((session_data, _)) = &existing {
            session.session_id = Some(session_data.session_id.clone());
            session.current_path = Some(canonical_path.clone());
            session.history = session_data.history.clone();

            let ts = chrono::Local::now().format("%H:%M:%S");
            println!("  [{ts}] ▶ Session restored: {canonical_path}");
            response_lines.push(format!("Session restored at `{}`.", canonical_path));
            response_lines.push(String::new());

            // Show last 5 conversation items
            let history_len = session_data.history.len();
            let start_idx = history_len.saturating_sub(5);
            for item in &session_data.history[start_idx..] {
                let prefix = match item.item_type {
                    HistoryType::User => "You",
                    HistoryType::Assistant => "AI",
                    HistoryType::Error => "Error",
                    HistoryType::System => "System",
                    HistoryType::ToolUse => "Tool",
                    HistoryType::ToolResult => "Result",
                };
                // Truncate long items for display
                let content: String = item.content.chars().take(200).collect();
                let truncated = if item.content.chars().count() > 200 {
                    "..."
                } else {
                    ""
                };
                response_lines.push(format!("[{}] {}{}", prefix, content, truncated));
            }
        } else {
            session.session_id = None;
            session.current_path = Some(canonical_path.clone());
            session.history.clear();

            let ts = chrono::Local::now().format("%H:%M:%S");
            println!("  [{ts}] ▶ Session started: {canonical_path}");
            response_lines.push(format!("Session started at `{}`.", canonical_path));
        }
    }

    // Persist chat_id -> path mapping for auto-restore after restart
    {
        let mut data = state.lock().await;
        data.settings
            .last_sessions
            .insert(chat_id.0.to_string(), canonical_path);
        save_bot_settings(token, &data.settings);
    }

    let response_text = response_lines.join("\n");
    send_long_message(bot, chat_id, &response_text, None, state).await?;

    Ok(())
}

/// Handle /clear command
pub(crate) async fn handle_clear_command(
    bot: &Bot,
    chat_id: ChatId,
    state: &SharedState,
) -> ResponseResult<()> {
    // Cancel in-progress AI request if any
    let cancel_token = {
        let data = state.lock().await;
        data.cancel_tokens.get(&chat_id).cloned()
    };
    if let Some(token) = cancel_token {
        token.cancelled.store(true, Ordering::Relaxed);
        if let Ok(guard) = token.child_pid.lock() {
            if let Some(pid) = *guard {
                #[cfg(unix)]
                // SAFETY: pid was obtained from child.id() and the child process is still
                // tracked. SIGTERM is a safe signal that asks the process to terminate.
                #[allow(unsafe_code)]
                unsafe {
                    libc::kill(pid as libc::pid_t, libc::SIGTERM);
                }
            }
        }
    }

    {
        let mut data = state.lock().await;
        if let Some(session) = data.sessions.get_mut(&chat_id) {
            session.session_id = None;
            session.history.clear();
            session.pending_uploads.clear();
            session.cleared = true;
        }
        data.cancel_tokens.remove(&chat_id);
        data.stop_message_ids.remove(&chat_id);
    }

    shared_rate_limit_wait(state, chat_id).await;
    bot.send_message(chat_id, "Session cleared.").await?;

    Ok(())
}

/// Handle /pwd command - show current session path
pub(crate) async fn handle_pwd_command(
    bot: &Bot,
    chat_id: ChatId,
    state: &SharedState,
) -> ResponseResult<()> {
    let current_path = {
        let data = state.lock().await;
        data.sessions
            .get(&chat_id)
            .and_then(|s| s.current_path.clone())
    };

    shared_rate_limit_wait(state, chat_id).await;
    match current_path {
        Some(path) => bot.send_message(chat_id, &path).await?,
        None => {
            bot.send_message(chat_id, "No active session. Use /start <path> first.")
                .await?
        }
    };

    Ok(())
}

/// Handle /cd command - change working directory without resetting session
pub(crate) async fn handle_cd_command(
    bot: &Bot,
    chat_id: ChatId,
    text: &str,
    state: &SharedState,
    token: &str,
) -> ResponseResult<()> {
    let path_str = text.strip_prefix("/cd").unwrap_or("").trim();

    // No argument: show current path (like /pwd)
    if path_str.is_empty() {
        let current_path = {
            let data = state.lock().await;
            data.sessions
                .get(&chat_id)
                .and_then(|s| s.current_path.clone())
        };
        shared_rate_limit_wait(state, chat_id).await;
        match current_path {
            Some(path) => {
                bot.send_message(chat_id, format!("Current: {path}"))
                    .await?
            }
            None => {
                bot.send_message(chat_id, "No active session. Use /start <path> first.")
                    .await?
            }
        };
        return Ok(());
    }

    // Expand ~ to home directory
    let expanded = if path_str.starts_with("~/") || path_str == "~" {
        if let Some(home) = dirs::home_dir() {
            home.join(path_str.strip_prefix("~/").unwrap_or(""))
                .display()
                .to_string()
        } else {
            path_str.to_string()
        }
    } else if path_str.starts_with('/') {
        path_str.to_string()
    } else {
        // Relative path: resolve against current_path
        let base = {
            let data = state.lock().await;
            data.sessions
                .get(&chat_id)
                .and_then(|s| s.current_path.clone())
        };
        match base {
            Some(b) => Path::new(&b).join(path_str).display().to_string(),
            None => {
                shared_rate_limit_wait(state, chat_id).await;
                bot.send_message(chat_id, "No active session. Use /start <path> first.")
                    .await?;
                return Ok(());
            }
        }
    };

    // Validate path
    let path = Path::new(&expanded);
    if !path.exists() || !path.is_dir() {
        shared_rate_limit_wait(state, chat_id).await;
        bot.send_message(chat_id, format!("Error: not a valid directory: {expanded}"))
            .await?;
        return Ok(());
    }

    let canonical = path
        .canonicalize()
        .map(|p| p.display().to_string())
        .unwrap_or(expanded);

    // Sandbox check: target must be within the home directory
    let sandbox_root = dirs::home_dir().unwrap_or_else(|| Path::new("/").to_path_buf());
    if !is_path_within_sandbox(Path::new(&canonical), &sandbox_root) {
        shared_rate_limit_wait(state, chat_id).await;
        bot.send_message(
            chat_id,
            format!(
                "Access denied: '{}' is outside the allowed path sandbox.",
                canonical
            ),
        )
        .await?;
        return Ok(());
    }

    // Update only current_path, preserve session and history
    {
        let mut data = state.lock().await;
        if let Some(session) = data.sessions.get_mut(&chat_id) {
            session.current_path = Some(canonical.clone());
        } else {
            shared_rate_limit_wait(state, chat_id).await;
            bot.send_message(chat_id, "No active session. Use /start <path> first.")
                .await?;
            return Ok(());
        }
    }

    // Persist the new path for auto-restore after bot restart (same as /start)
    {
        let mut data = state.lock().await;
        data.settings
            .last_sessions
            .insert(chat_id.0.to_string(), canonical.clone());
        save_bot_settings(token, &data.settings);
    }

    shared_rate_limit_wait(state, chat_id).await;
    bot.send_message(chat_id, format!("Changed to: {canonical}"))
        .await?;

    Ok(())
}

/// Handle /stop command - cancel in-progress AI request
pub(crate) async fn handle_stop_command(
    bot: &Bot,
    chat_id: ChatId,
    state: &SharedState,
) -> ResponseResult<()> {
    let token = {
        let data = state.lock().await;
        data.cancel_tokens.get(&chat_id).cloned()
    };

    match token {
        Some(token) => {
            // Ignore duplicate /stop if already cancelled
            if token.cancelled.load(Ordering::Relaxed) {
                return Ok(());
            }

            // Send immediate feedback to user
            shared_rate_limit_wait(state, chat_id).await;
            let stop_msg = bot.send_message(chat_id, "Stopping...").await?;

            // Store the stop message ID so the polling loop can update it later
            {
                let mut data = state.lock().await;
                data.stop_message_ids.insert(chat_id, stop_msg.id);
            }

            // Set cancellation flag
            token.cancelled.store(true, Ordering::Relaxed);

            // Kill child process directly to unblock reader.lines()
            // When the child dies, its stdout pipe closes -> reader returns EOF -> blocking thread exits
            if let Ok(guard) = token.child_pid.lock() {
                if let Some(pid) = *guard {
                    #[cfg(unix)]
                    // SAFETY: pid was obtained from child.id() and the child process is still
                    // tracked. SIGTERM is a safe signal that asks the process to terminate.
                    #[allow(unsafe_code)]
                    unsafe {
                        libc::kill(pid as libc::pid_t, libc::SIGTERM);
                    }
                }
            }

            let ts = chrono::Local::now().format("%H:%M:%S");
            println!("  [{ts}] ■ Cancel signal sent");
        }
        None => {
            shared_rate_limit_wait(state, chat_id).await;
            bot.send_message(chat_id, "No active request to stop.")
                .await?;
        }
    }

    Ok(())
}

/// Handle /public command - toggle public access for group chats
pub(crate) async fn handle_public_command(
    bot: &Bot,
    chat_id: ChatId,
    text: &str,
    state: &SharedState,
    token: &str,
    is_group_chat: bool,
    is_owner: bool,
) -> ResponseResult<()> {
    if !is_group_chat {
        shared_rate_limit_wait(state, chat_id).await;
        bot.send_message(chat_id, "This command is only available in group chats.")
            .await?;
        return Ok(());
    }

    if !is_owner {
        shared_rate_limit_wait(state, chat_id).await;
        bot.send_message(
            chat_id,
            "Only the bot owner can change public access settings.",
        )
        .await?;
        return Ok(());
    }

    let arg = text
        .strip_prefix("/public")
        .unwrap_or("")
        .trim()
        .to_lowercase();
    let chat_key = chat_id.0.to_string();

    let response_msg = match arg.as_str() {
        "on" => {
            let mut data = state.lock().await;
            data.settings
                .as_public_for_group_chat
                .insert(chat_key, true);
            save_bot_settings(token, &data.settings);
            "✅ Public access <b>enabled</b> for this group.\nAll members can now use the bot."
                .to_string()
        }
        "off" => {
            let mut data = state.lock().await;
            data.settings.as_public_for_group_chat.remove(&chat_key);
            save_bot_settings(token, &data.settings);
            "❌ Public access <b>disabled</b> for this group.\nOnly the owner can use the bot."
                .to_string()
        }
        "" => {
            let data = state.lock().await;
            let is_public = data
                .settings
                .as_public_for_group_chat
                .get(&chat_key)
                .copied()
                .unwrap_or(false);
            let status = if is_public { "enabled" } else { "disabled" };
            format!(
                "Public access is currently <b>{}</b> for this group.\n\n\
                 <code>/public on</code> — Allow all members\n\
                 <code>/public off</code> — Owner only",
                status
            )
        }
        _ => {
            "Usage:\n<code>/public on</code> — Allow all group members\n<code>/public off</code> — Owner only".to_string()
        }
    };

    shared_rate_limit_wait(state, chat_id).await;
    bot.send_message(chat_id, &response_msg)
        .parse_mode(ParseMode::Html)
        .await?;

    Ok(())
}

/// Auto-restore session from bot_settings.json if not in memory.
/// If there is no previous path, fall back to startup project dir.
pub(crate) fn auto_restore_session(
    data: &mut super::bot::SharedData,
    chat_id: ChatId,
    default_project_dir: &str,
    user_name: &str,
) {
    if !data.sessions.contains_key(&chat_id) {
        let candidate_path = data
            .settings
            .last_sessions
            .get(&chat_id.0.to_string())
            .cloned()
            .unwrap_or_else(|| default_project_dir.to_string());
        if Path::new(&candidate_path).is_dir() {
            let existing = load_existing_session(&candidate_path);
            let session = data.sessions.entry(chat_id).or_insert_with(|| ChatSession {
                session_id: None,
                current_path: None,
                history: Vec::new(),
                pending_uploads: Vec::new(),
                cleared: false,
            });
            session.current_path = Some(candidate_path.clone());
            if let Some((session_data, _)) = existing {
                session.session_id = Some(session_data.session_id.clone());
                session.history = session_data.history.clone();
            }
            let ts = chrono::Local::now().format("%H:%M:%S");
            println!("  [{ts}] ↻ [{user_name}] Auto-restored session: {candidate_path}");
        }
    }
}
