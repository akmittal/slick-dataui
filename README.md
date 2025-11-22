# Slick DataUI

Slick DataUI is a modern, cross-platform database client built with Rust and GPUI. It provides a high-performance, native desktop experience for managing SQLite and PostgreSQL databases.

## Features

### 🔌 Connection Management
- **Multi-Database Support**: Connect to SQLite and PostgreSQL databases.
- **Connection Manager**: Easily add, save, and manage multiple database connections.
- **Visual Interface**: Intuitive sidebar for quick access to your connections.

### 🗄️ Schema Browsing
- **Table Introspection**: Automatically fetches and displays tables upon connection.
- **Sidebar Navigation**: Browse database structure directly from the sidebar.

### 📝 Query Execution
- **SQL Editor**: Integrated query editor for writing and executing SQL commands.
- **Async Execution**: Non-blocking query execution ensures the UI remains responsive.
- **Results Grid**: View query results in a structured table format.

## Technology Stack

- **Language**: [Rust](https://www.rust-lang.org/) 🦀
- **UI Framework**: [GPUI](https://github.com/zed-industries/zed/tree/main/crates/gpui) (The same framework powering the Zed editor)
- **Database Interface**: [SQLx](https://github.com/launchbadge/sqlx) for async, type-safe database interactions.
- **Async Runtime**: [Tokio](https://tokio.rs/) (for database operations)
- **File Dialogs**: [rfd](https://github.com/PolyMeilex/rfd) for native file picker dialogs

## Getting Started

### Prerequisites
- Rust and Cargo (latest stable version)
- SQLite / PostgreSQL (for testing connections)

### Installation

Clone the repository:
```bash
git clone https://github.com/yourusername/slick-dataui.git
cd slick-dataui
```

### Running the Application

```bash
cargo run
```

## Usage Guide

1. **Add a Connection**:
   - Click the **+** button in the "Connections" sidebar.
   - Enter a name for your connection.
   - Select the database type (SQLite or PostgreSQL) using the radio buttons.
   - **For SQLite**: Click the **Browse...** button to open a file picker and select your database file. The connection string will be automatically populated.
   - **For PostgreSQL**: Manually enter the connection string (e.g., `postgres://user:password@host/dbname`).
   - Click **Save**.

2. **Connect**:
   - Click on the connection name in the sidebar.
   - The app will connect and list the tables.

3. **Run a Query**:
   - Type your SQL query in the "Query Editor" pane.
   - Click **Run**.
   - Results will appear in the "Results" pane below.

## Development

### Project Structure
- `src/main.rs`: Application entry point and setup.
- `src/ui.rs`: UI components, layout, and event handling.
- `src/state.rs`: Application state management (Redux-like store).
- `src/db.rs`: Database abstraction layer and client implementations.

### Building
```bash
cargo build --release
```

## Contributing

### AI Agents
If you are an AI agent contributing to this project, please read [AI_GUIDELINES.md](AI_GUIDELINES.md) for architectural patterns and coding standards.
