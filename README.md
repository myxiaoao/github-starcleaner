# GitHub Star Cleaner

A native macOS desktop application for batch managing and cleaning up your GitHub starred repositories.

## Features

- **Browse Starred Repos**: View all your GitHub starred repositories with details (stars, forks, language, description, last push time)
- **Sorting**: Sort repositories by starred time or last push time (ascending/descending)
- **Batch Operations**: Select multiple repositories and unstar them in batch
- **Search/Filter**: Filter repositories by name, description, language, or topics
- **Clickable Links**: Click repository names to open them in your browser
- **Confirmation Dialogs**: All destructive operations require confirmation
- **Pagination**: Load more repositories on demand (100 per page)

## Screenshots

*Coming soon*

## Requirements

- macOS (uses GPUI framework with Metal rendering)
- GitHub Personal Access Token with `repo` and `user` scopes

## Installation

### From Source

1. Make sure you have Rust installed (https://rustup.rs/)

2. Clone the repository:
   ```bash
   git clone https://github.com/yourusername/github-starcleaner.git
   cd github-starcleaner
   ```

3. Build and run:
   ```bash
   cargo run --release
   ```

## Setup

1. Create a GitHub Personal Access Token:
   - Go to GitHub Settings > Developer settings > Personal access tokens > Tokens (classic)
   - Generate a new token with the following scopes:
     - `repo` (for repository access)
     - `user` (for starring/unstarring)

2. Launch the application and paste your token in the setup screen

3. Your token will be stored locally in `~/.config/github-starcleaner/config.toml`

## Usage

### Sorting

- Click **Starred** to sort by when you starred the repository
- Click **Pushed** to sort by when the repository was last pushed to
- Click the direction indicator (↑/↓) to toggle ascending/descending order
- Default: Pushed ascending (oldest push first - helps find inactive repos)

### Selecting Repositories

- Click the checkbox next to any repository to select it
- Use "Select All" in the toolbar to select all visible repositories
- Click "Unstar Selected (N)" to batch unstar selected repositories

### Unstarring

- Click the "Unstar" button on any repository row to unstar a single repo
- All unstar operations show a confirmation dialog before proceeding

## Project Structure

```
src/
├── main.rs              # Application entry point
├── lib.rs               # Library exports
├── models/              # Data models
│   ├── mod.rs
│   ├── config.rs        # App configuration
│   └── repository.rs    # Repository model
├── services/            # Business logic
│   ├── mod.rs
│   ├── config.rs        # Config file management
│   └── github.rs        # GitHub API service
├── state/               # Application state
│   ├── mod.rs
│   └── app_state.rs     # Global app state
└── ui/                  # UI components
    ├── mod.rs
    ├── app_view.rs      # Main application view
    ├── setup_view.rs    # Token setup screen
    ├── repository_list.rs  # Repository list view
    └── repository_row.rs   # Single repository row
```

## Tech Stack

- **[GPUI](https://gpui.rs/)** - High-performance native UI framework (from Zed editor)
- **[Octocrab](https://github.com/XAMPPRocky/octocrab)** - GitHub API client for Rust
- **[Tokio](https://tokio.rs/)** - Async runtime
- **[Serde](https://serde.rs/)** - Serialization/deserialization

## License

MIT
