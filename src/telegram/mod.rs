mod bot;
mod commands;
mod file_ops;
mod message;
mod storage;
mod streaming;
mod tools;

use std::collections::HashMap;

use teloxide::prelude::*;

use self::bot::SharedData;
use self::message::handle_message;
use self::storage::load_bot_settings;

// Re-export public API used by main.rs
pub use self::storage::resolve_token_by_hash;
pub use self::storage::token_hash;

/// Entry point: start the Telegram bot with long polling.
/// `default_project_dir` is the working directory bound by the CLI binary.
pub async fn run_bot(token: &str, default_project_dir: &str) {
    let bot = Bot::new(token);
    let bot_settings = load_bot_settings(token);

    // Register bot commands for autocomplete
    let commands = vec![
        teloxide::types::BotCommand::new("help", "Show help"),
        teloxide::types::BotCommand::new("start", "Start session at directory"),
        teloxide::types::BotCommand::new("pwd", "Show current working directory"),
        teloxide::types::BotCommand::new("cd", "Change working directory"),
        teloxide::types::BotCommand::new("clear", "Clear AI conversation history"),
        teloxide::types::BotCommand::new("stop", "Stop current AI request"),
        teloxide::types::BotCommand::new("down", "Download file from server"),
        teloxide::types::BotCommand::new("public", "Toggle public access (group only)"),
        teloxide::types::BotCommand::new("availabletools", "List all available tools"),
        teloxide::types::BotCommand::new("allowedtools", "Show currently allowed tools"),
        teloxide::types::BotCommand::new("allowed", "Add/remove tool (+name / -name)"),
    ];
    if let Err(e) = bot.set_my_commands(commands).await {
        println!("  ⚠ Failed to set bot commands: {e}");
    }

    match bot_settings.owner_user_id {
        Some(owner_id) => println!("  ✓ Owner: {owner_id}"),
        None => println!("  ⚠ No owner registered — first user will be registered as owner"),
    }

    let state: bot::SharedState = std::sync::Arc::new(tokio::sync::Mutex::new(SharedData {
        sessions: HashMap::new(),
        settings: bot_settings,
        cancel_tokens: HashMap::new(),
        stop_message_ids: HashMap::new(),
        api_timestamps: HashMap::new(),
    }));

    println!("  ✓ Bot connected — Listening for messages");

    let shared_state = state.clone();
    let token_owned = token.to_string();
    let default_project_dir_owned = default_project_dir.to_string();
    teloxide::repl(bot, move |bot: Bot, msg: Message| {
        let state = shared_state.clone();
        let token = token_owned.clone();
        let default_project_dir = default_project_dir_owned.clone();
        async move { handle_message(bot, msg, state, &token, &default_project_dir).await }
    })
    .await;
}
