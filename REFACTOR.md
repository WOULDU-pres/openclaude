# Refactoring: telegram.rs Module Decomposition

## Overview

This document tracks the refactoring of `src/telegram.rs` (2642 LOC) into a modular structure at `src/telegram/`.

**Status**: In Progress (Task #2)
**Target**: Zero behavior change, improved maintainability

## Target Structure

```
src/telegram/
├── mod.rs              (~100 LOC)  - Module re-exports, run_bot()
├── bot.rs             (~300 LOC)  - BotSettings, ChatSession, initialization
├── commands.rs        (~500 LOC)  - /help, /start, /cd, /pwd, /stop, /clear, /public
├── file_ops.rs        (~200 LOC)  - /down handler, upload handler
├── tools.rs           (~200 LOC)  - /allowedtools, /availabletools, /allowed +/-
├── message.rs         (~600 LOC)  - Text message handler, AI invocation
├── streaming.rs       (~400 LOC)  - Streaming response, spinner, HTML conversion
└── storage.rs         (~200 LOC)  - Session save/load, settings file I/O
```

## Module Dependencies

Decomposition order (build order respects dependencies):

1. **storage.rs** (no internal deps)
   - `save_session()`, `load_session()`, `save_bot_settings()`, `load_bot_settings()`
   - Responsible for all I/O to `~/.openclaude/` directory

2. **bot.rs** (depends on storage)
   - `BotSettings`, `ChatSession`, `SharedState` struct definitions
   - `initialize_bot()`, state initialization
   - Per-chat worker setup

3. **tools.rs** (depends on bot)
   - Tool allowlist logic
   - `/availabletools`, `/allowedtools`, `/allowed +/-toolname` handlers

4. **file_ops.rs** (depends on bot + storage)
   - `/down <file>` handler
   - File upload processing
   - File delivery command generation

5. **streaming.rs** (independent)
   - Streaming response handling
   - Spinner animation
   - HTML formatting for Telegram

6. **commands.rs** (depends on bot + storage + file_ops)
   - `/help`, `/start`, `/cd`, `/pwd`, `/stop`, `/clear`, `/public` handlers
   - Session-level commands

7. **message.rs** (depends on all - last)
   - `handle_message()` - main router
   - Text message handler
   - AI invocation pipeline
   - Requires all other modules to compile

8. **mod.rs** (re-exports)
   - `pub use crate::telegram::*` patterns
   - `pub fn run_bot()` entry point
   - Maintains public API compatibility

## Public API Contract

### Exports from `src/telegram/mod.rs`

These must remain `pub` to keep `src/main.rs` working:

```rust
pub async fn run_bot(...) -> ResponseResult<()>
pub struct BotSettings { ... }
pub struct ChatSession { ... }
pub struct SharedState { ... }
pub type ResponseResult<T> = Result<T, RequestError>;
```

## Command Handler Organization

| Command | Module | Handler Function |
|---------|--------|------------------|
| `/help` | commands.rs | `handle_help_command()` |
| `/start [path]` | commands.rs | `handle_start_command()` |
| `/pwd` | commands.rs | `handle_pwd_command()` |
| `/cd <path>` | commands.rs | `handle_cd_command()` |
| `/stop` | commands.rs | `handle_stop_command()` |
| `/clear` | commands.rs | `handle_clear_command()` |
| `/down <file>` | file_ops.rs | `handle_down_command()` |
| `/public on/off` | commands.rs | `handle_public_command()` |
| `/availabletools` | tools.rs | `handle_availabletools_command()` |
| `/allowedtools` | tools.rs | `handle_allowedtools_command()` |
| `/allowed +/-name` | tools.rs | `handle_allowed_command()` |
| `!<shell>` | message.rs | `handle_shell_command()` |
| `;<message>` | message.rs | (handled in main flow) |
| Text messages | message.rs | `handle_text_message()` |

## Implementation Strategy

### Phase 1: Extract Leaf Modules
- Extract storage.rs first (no deps, simplest)
- Extract bot.rs (depends only on storage)
- Extract tools.rs
- Extract file_ops.rs

### Phase 2: Extract Dependent Modules
- Extract streaming.rs (independent, can be anytime)
- Extract commands.rs (depends on bot + storage + file_ops)

### Phase 3: Final Integration
- Extract message.rs (depends on all)
- Update mod.rs with re-exports
- Update main.rs imports if needed
- Run tests and verification

## Key Refactoring Rules

1. **No Behavior Change**: All functionality must work identically after refactoring
2. **Preserve Public API**: Keep all public structs and functions accessible from `src/main.rs`
3. **Update Imports**: All `use crate::claude::` statements must be correct in each module
4. **Compilation Order**: Dependencies must respect the order listed above
5. **Test Preservation**: All 22 existing tests must pass

## Testing Plan

After refactoring completion:

```bash
# Build test
cargo build --release

# Test all 22 tests
cargo test

# Type checking
cargo check

# Linting
cargo clippy
```

## Documentation Updates Post-Refactor

After Task #2 completion, update:
- This REFACTOR.md with completion date
- API.md with new module organization
- README.md if needed (likely no changes required)

## Rollback Plan

If issues arise:
1. Revert to previous git commit
2. Return to original telegram.rs
3. Investigate issue in isolation
4. Restart refactoring with fix

## Success Criteria

- ✅ All 2642 LOC extracted to modular structure
- ✅ No behavior changes
- ✅ All 22 tests pass
- ✅ cargo check/build/clippy succeed
- ✅ Public API unchanged
- ✅ Documentation updated
