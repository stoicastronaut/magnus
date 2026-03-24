# Magnus — Claude Desktop Alternative

An open-source Claude desktop application built with Tauri (Rust + React) that supports proxy connections and multi-agent management.

## Project Goals

- Proxy support for corporate environments (HTTP/HTTPS/SOCKS5)
- Multi-agent management (multiple Claude sessions/agents)
- Open-source alternative to Claude Desktop
- Lightweight native binary (not Electron)

## Tech Stack

- **Framework**: Tauri v2 (Rust backend + React frontend)
- **Frontend**: React + TypeScript + Vite
- **UI Library**: TBD
- **State Management**: TBD
- **Package Manager**: pnpm (preferred)
- **Rust edition**: 2021

## Architecture Principles

### Rust Backend (src-tauri/)
- All Claude API calls go through Rust using `reqwest` — this is what enables proxy support
- Tauri commands (`#[tauri::command]`) are the public interface exposed to the frontend
- Keep commands thin: they receive input, call a service, return output
- Business logic lives in modules, not in `main.rs` or `lib.rs` directly
- Use `thiserror` for typed errors, `anyhow` for internal error chaining

### React Frontend (src/)
- Frontend is purely a UI layer — it never calls Claude API directly
- Communicate with Rust via `@tauri-apps/api/core` `invoke()` calls
- Keep Tauri-specific code in a dedicated service/adapter layer (e.g., `src/services/tauri.ts`)
  so the UI components stay portable and testable
- Prefer small, focused components

### Multi-Agent Design
- Each agent is an isolated conversation context with its own state
- Agents can run concurrently — design for this from the start
- Agent state is persisted locally (Tauri's app data directory)

## Key Rust Crates to Know

| Crate | Purpose |
|-------|---------|
| `tauri` | Core framework |
| `reqwest` | HTTP client with proxy support |
| `serde` / `serde_json` | Serialization (Tauri commands use this) |
| `tokio` | Async runtime |
| `thiserror` | Typed error definitions |
| `anyhow` | Error chaining for internal use |

## Key Tauri Concepts

- **Commands**: Rust functions exposed to the frontend via `invoke()`
- **Events**: Async messages from Rust to Frontend (good for streaming responses)
- **Permissions**: Declared in `tauri.conf.json` — follow least-privilege
- **App Data Dir**: Use `tauri::Manager::app_data_dir()` for persistent storage

## Proxy Support Strategy

Proxy configuration is passed to `reqwest::Client` at construction time.
The client is stored as Tauri `State` and shared across commands.
Users configure proxy settings in the UI; settings are persisted locally.

```
Frontend (proxy settings UI)
  -> invoke("set_proxy_config", { url, username, password })
  -> Rust: rebuild reqwest::Client with new proxy config
  -> Store updated client in app State
```

## Development Commands

```bash
# Start dev server (hot reload for frontend, recompiles Rust on change)
pnpm tauri dev

# Build production binary
pnpm tauri build

# Run frontend only (no Rust)
pnpm dev

# Type-check
pnpm tsc --noEmit
```

## Project Structure

```
magnus/
├── src/                        # React frontend
│   ├── components/             # UI components
│   ├── services/               # Tauri invoke wrappers
│   ├── stores/                 # State management
│   ├── App.tsx
│   └── main.tsx
├── src-tauri/                  # Rust backend
│   ├── src/
│   │   ├── main.rs             # Tauri app setup only
│   │   ├── lib.rs              # Command registration
│   │   ├── claude/             # Claude API client module
│   │   ├── agents/             # Agent management
│   │   └── config/             # Proxy + app config
│   ├── Cargo.toml
│   └── tauri.conf.json
├── CLAUDE.md
├── package.json
└── vite.config.ts
```

## Conventions

- Rust: follow `rustfmt` defaults, use `clippy` warnings as errors in CI
- TypeScript: strict mode enabled
- Name Tauri commands in `snake_case` (Rust) — they map to camelCase in the frontend by convention
- Errors returned from commands should be `String` (serializable) at the command boundary

## Learning Resources

- Tauri v2 docs: https://v2.tauri.app
- Tauri commands guide: https://v2.tauri.app/develop/calling-rust/
- reqwest proxy docs: https://docs.rs/reqwest/latest/reqwest/struct.Proxy.html
- Tauri state management: https://v2.tauri.app/develop/state-management/
