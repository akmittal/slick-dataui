# AI Guidelines for Slick DataUI

This document provides instructions and patterns for AI agents contributing to the Slick DataUI repository.

## Project Overview
Slick DataUI is a Rust-based database client using the GPUI framework. It aims to provide a high-performance, native experience for managing SQLite and PostgreSQL databases.

## Architecture

### Frontend (GPUI)
- **Framework**: [GPUI](https://github.com/zed-industries/zed/tree/main/crates/gpui).
- **Components**: Use `gpui-component` for standard UI elements (buttons, inputs, tables).
- **State Management**:
    - `AppState`: The core application state struct.
    - `GlobalAppState`: A wrapper around `Entity<AppState>` to pass state through the view hierarchy.
    - **Pattern**: Pass `GlobalAppState` to views. Use `cx.observe` or `cx.subscribe` to react to state changes.

### Backend (Database)
- **Interface**: `sqlx` (async).
- **Traits**: `DatabaseClient` trait abstracts specific DB implementations (SQLite, Postgres).
- **Async Runtime**: `tokio`.

## Coding Standards

### Rust & Async
- **Runtime**: The application runs on a global `tokio` runtime.
- **Async/Await**:
    - **DO NOT** use `block_on` inside `async` functions. This causes "runtime within runtime" panics.
    - Use `.await` for all async operations.
    - If you need to bridge sync code to async, use `cx.spawn` or `cx.background_executor().spawn`.

### Error Handling
- Use `anyhow::Result` for functions that can fail.
- Propagate errors up to the UI layer where they can be displayed to the user.

### UI Patterns
- **Delegates**: Use delegates (e.g., `TableDelegate`) for complex UI logic like tables.
- **Styling**: Use GPUI's fluent styling API (e.g., `.flex()`, `.bg()`, `.text_color()`).

## Testing Patterns

### Unit Tests
- Place unit tests in a `#[cfg(test)] mod tests` module within the source file.
- **Async Tests**:
    - Use `#[test]` but wrap async code in a runtime block if needed, OR use `#[tokio::test]` if the crate supports it (currently using manual `block_on` in tests is common pattern here, but be careful not to nest runtimes if the test harness provides one).
    - *Correction*: The project uses `once_cell` to define a global `TOKIO_RUNTIME`. In tests, use `TOKIO_RUNTIME.block_on(async { ... })` for top-level test functions, but ensure the code under test does NOT call `block_on` internally.

### Database Tests
- **SQLite**: Use in-memory databases (`sqlite::memory:`) for fast, isolated tests.
- **Mocking**: Avoid heavy mocking frameworks; prefer using the real (in-memory) DB or simple traits.

## Common Tasks

### Adding a New Feature
1. **State**: Add necessary fields to `AppState`.
2. **DB**: Update `DatabaseClient` trait and implementations if new DB ops are needed.
3. **UI**: Create a new view or update existing ones. Connect UI actions to `AppState` methods.

### Debugging
- Check `cargo test` output.
- Look for "runtime within runtime" panics if async code crashes.
