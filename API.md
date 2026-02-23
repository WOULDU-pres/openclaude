# openclaude API Documentation

## Overview

`openclaude` is a Rust CLI that bridges Telegram and Claude Code. This document describes the internal API structure.

## Core Modules

### `main.rs`
- **Purpose**: CLI entry point and server initialization
- **Responsibilities**:
  - Parse command-line arguments (project directory, token, madmax flag)
  - Initialize Telegram bot connection
  - Start the main message polling loop

### `telegram.rs` (2642 LOC - Target for Refactoring)
- **Purpose**: Telegram message handling and command routing
- **Key Structures**:
  - `SharedState`: Arc-wrapped state containing sessions, cancel tokens, rate limit trackers
  - `ChatSession`: Per-chat conversation state with history and file uploads

- **Command Handlers**:
  - `/help` - Display help text
  - `/start [path]` - Initialize session at directory
  - `/pwd` - Show current working directory
  - `/cd <path>` - Change working directory (session-preserving)
  - `/clear` - Clear conversation history
  - `/stop` - Cancel in-progress AI request
  - `/down <file>` - Download file to user
  - `/public on|off` - Toggle group chat access
  - `/availabletools` - List available tools
  - `/allowedtools` - Show enabled tools
  - `/allowed +/-toolname` - Add/remove tool
  - `!<command>` - Execute shell command
  - `;<message>` - Send message in group chat
  - Text messages - Pass to Claude Code for AI response

- **Features**:
  - Per-chat message polling (teloxide workers)
  - Rate limiting with exponential backoff
  - Session persistence to `~/.openclaude/sessions/*.json`
  - File upload handling with pending queue
  - Streaming response support

### `claude.rs`
- **Purpose**: Claude Code CLI interaction
- **Responsibilities**:
  - Execute Claude Code commands with spawn_blocking
  - Handle streaming responses
  - Manage session IDs and history

### `auth.rs`
- **Purpose**: Permission and authentication model
- **Key Features**:
  - Owner imprinting on first DM
  - Group chat access control (/public command)
  - Tool allowlist management
  - madmax override mode

### `session.rs`
- **Purpose**: Session state management
- **Responsibilities**:
  - Load/save session JSON files
  - Manage conversation history
  - Track pending file uploads

### `app.rs`
- **Purpose**: Application lifecycle and configuration
- **Responsibilities**:
  - Bot settings persistence
  - Token validation
  - Configuration directory management

## Data Flow

1. **Message Reception**: Telegram bot receives message
2. **Routing**: `handle_message()` routes to appropriate command handler
3. **Command Execution**: Handler processes command and updates state
4. **Response**: Handler sends response back to Telegram
5. **Session Update**: Session state persisted to disk

## Key Design Patterns

### Per-Chat Workers
Teloxide spawns a separate async worker for each chat, allowing per-chat command processing without blocking other conversations.

### Rate Limiting
Exponential backoff per chat prevents Telegram API throttling.

### Session Persistence
All conversations automatically saved to `~/.openclaude/sessions/<chat_id>.json` for recovery.

### Tool Allowlist
Users can enable/disable Claude Code tools (Bash, Python, etc.) via `/allowed` command.

## Testing Entry Points

- `telegram.rs::handle_message()` - Main message router
- `claude.rs::execute_claude()` - Claude Code execution
- `auth.rs::check_permission()` - Permission checks
- `session.rs::save_session()` - Persistence

## Performance Considerations

- **telegram.rs size**: 2642 LOC suggests opportunity for modular refactoring
- **Polling overhead**: Per-chat workers may scale with group chat count
- **Session persistence**: Synchronous file I/O, consider async for high-volume scenarios
- **Rate limiting**: Current backoff may be conservative for large user bases

## Known Issues

See README.md "Known Issues" section for dependency vulnerabilities.
