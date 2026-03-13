# Habits

A lightweight desktop productivity tracker that monitors your active and idle time, logs work sessions, and shows per-app usage — all stored locally with no cloud dependency.

Built with **Tauri 2**, **React 19**, **TypeScript**, and **Rust** (SQLite backend).

---

## Features

- **Live status badge** — instantly shows whether you are productive or idle
- **Dashboard** — real-time session timer with today's productive / idle totals
- **History** — daily summaries (productive vs. idle hours per day)
- **Sessions** — full list of recorded work sessions with start/end times
- **App usage** — per-application time breakdown for the current day
- **Settings** — configurable idle threshold and optional autostart on login
- **System-tray icon** — app stays accessible without cluttering the taskbar
- **Local SQLite database** — all data lives on-device in your app-data folder

---

## Tech Stack

| Layer | Technology |
|-------|-----------|
| Desktop shell | Tauri 2 |
| Frontend | React 19, TypeScript 5, Vite 7 |
| Styling | Tailwind CSS 4 |
| Backend | Rust (rusqlite / SQLite) |
| Package manager | Bun |

---

## Prerequisites

| Tool | Version |
|------|---------|
| [Node.js](https://nodejs.org/) | 18+ |
| [Bun](https://bun.sh/) | latest |
| [Rust](https://www.rust-lang.org/tools/install) | stable (via `rustup`) |
| Tauri prerequisites | see [Tauri v2 prerequisites](https://tauri.app/start/prerequisites/) |

---

## Getting Started

```bash
# Install JS dependencies
bun install

# Start the app in development mode (Vite dev server + Tauri window)
bun run tauri dev
```

### Available scripts

| Command | Description |
|---------|-------------|
| `bun run dev` | Vite dev server only (no Tauri window) |
| `bun run build` | Type-check and build the frontend |
| `bun run tauri dev` | Full Tauri development build with HMR |
| `bun run tauri build` | Production desktop bundle |

---

## Project Structure

```
src/                  # React / TypeScript frontend
  components/         # UI components (Dashboard, History, Sessions, …)
  hooks/              # useTracker — polls Rust backend every 10 s
  api.ts              # Tauri command bindings
  types.ts            # Shared TypeScript types mirroring Rust structs
src-tauri/
  src/                # Rust backend
    lib.rs            # App setup, tray icon, state wiring
    tracker.rs        # Idle / active state machine
    commands.rs       # Tauri commands exposed to the frontend
    db.rs             # SQLite schema, migrations, queries
    idle.rs           # System idle-time detection
    active_app.rs     # Foreground application detection
  tauri.conf.json     # App metadata and window config
  Cargo.toml
```

---

## How It Works

1. On startup the Rust backend opens (or creates) a SQLite database at the platform app-data directory.
2. A background tracker thread samples system idle time and records active vs. idle seconds per session.
3. The React frontend polls the backend every 10 seconds via Tauri commands and maintains a locally-ticking second counter for a smooth UI.
4. Sessions are automatically closed and written to the database when the app exits or a new session begins.

---

## Recommended IDE Setup

[VS Code](https://code.visualstudio.com/) with the following extensions:

- [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode)
- [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)
- [ESLint](https://marketplace.visualstudio.com/items?itemName=dbaeumer.vscode-eslint)

---

## License

See [LICENSE.txt](LICENSE.txt).
