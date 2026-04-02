# Magnus — Claude Desktop Alternative

## Working Mode

The developer is **learning Rust** through this project. Claude's role is pair-programming mentor for all Rust code:
- **Never write Rust code unless explicitly asked to**
- Explain concepts, guide implementation, review written code, and point out errors
- Ask the developer to attempt the code first, then give feedback
- React/TypeScript code can be written directly by Claude when asked



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
- **Package Manager**: pnpm (preferred) — but `cargo tauri dev` is used instead of `pnpm tauri dev`
- **Rust edition**: 2021
- **Node**: must be native arm64 on Apple Silicon (not Rosetta x64)

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

### Frontend Component Architecture
Before modifying or adding any frontend feature, always plan the component structure first:
- Identify which components are affected and whether new ones are needed
- Each component should have a single responsibility (e.g. `Sidebar` handles navigation, `ChatArea` handles messages)
- State that is shared between components lives in the nearest common parent (usually a page-level component like `HomePage`)
- State that is local to a component (e.g. rename input value) stays inside that component
- Never grow a single component to handle multiple concerns — split it instead
- Page-level components (e.g. `HomePage`) orchestrate state and pass props down; they contain no JSX of their own beyond layout
- Keep views always mounted when their state needs to survive navigation (use `display: none` instead of conditional rendering)

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
cargo tauri dev          # preferred — avoids pnpm native binding issues

# Build production binary
cargo tauri build

# Run frontend only (no Rust)
pnpm dev

# Type-check
pnpm tsc --noEmit
```

> Note: `pnpm tauri dev` has native binding issues on this machine. Always use `cargo tauri dev` instead.
> Tauri CLI installed via: `cargo install tauri-cli --version "^2.0.0" --locked`

## Current State (as of end of session 3)

### What's implemented
- **Settings**: API key + base URL saved to disk as JSON (`~/Library/Application Support/com.magnus.app/settings.json`)
- **Chat**: Streaming chat via SSE, full conversation history sent on each request
- **Sidebar**: Left navbar with chat list, new chat button, right-click context menu (rename/delete), settings button at bottom
- **Multi-chat**: Multiple independent chat sessions, persisted to disk
- **Chat persistence**: Each chat saved as `{app_data_dir}/chats/{dd-mm-yy}-{uuid}.json`, loaded on startup
- **Proxy**: Works by setting a custom `base_url` in settings (e.g. `https://llm-proxy.edgez.live/`) — the Anthropic SDK convention of appending `v1/messages` to the base URL is followed

### Known Issues / Limitations
- Model is hardcoded to `claude-haiku-4-5-20251001` in `claude.rs`
- No system prompt support yet
- No proxy authentication support (HTTP/HTTPS proxy via reqwest not yet implemented)

### Proxy notes
The proxy used in development is `https://llm-proxy.edgez.live/` — it follows standard Anthropic API format. The request goes to `{base_url}v1/messages`. Note: no slash between base_url and `v1` since base_url already has a trailing slash.

## Current File Structure

```
src/
├── App.tsx                     # Routing — keeps HomePage always mounted to preserve state
├── components/
│   ├── HomePage.tsx            # State orchestration for chat sessions
│   ├── Sidebar.tsx             # Chat list, new chat, rename, settings button
│   ├── ChatArea.tsx            # Messages display + input
│   └── SettingsPage.tsx        # API key + base URL form

src-tauri/src/
├── main.rs                     # Entry point — calls lib::run()
├── lib.rs                      # Tauri command registration
├── config.rs                   # Settings struct, save/load to disk
├── claude.rs                   # Message struct, stream_message
└── chats.rs                    # Chat struct, save/load/delete per-file
```

## Future Features

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

The following features are planned but not yet implemented:

### 1. Multiple Models and Providers
Allow users to select the model per chat or globally.
- Add `model: String` to `Settings` (or per-chat config)
- Expose a model selector in the UI
- Support non-Anthropic providers (OpenAI-compatible APIs) by making the request format configurable

### 2. Multi-Agent Mode
Run multiple Claude agents concurrently with shared or independent context.
- Each agent has its own conversation thread and possibly a system prompt
- Agents can be orchestrated (one agent spawns sub-agents)
- UI: agent panel showing active agents and their status

### 3. Computer Task Manipulation
Allow Claude to interact with the local filesystem (move, copy, create, delete files).
- Expose Tauri commands for filesystem operations with permission prompts
- Connect to Claude's tool_use API feature
- Security: strict permission model, user must approve each action type

### 4. Automatic Chat Naming
Auto-generate a meaningful chat name based on the first message + response.
- After the first assistant response, invoke a secondary Claude call with a short prompt: "Summarize this conversation in 4 words"
- Update the chat name in the sidebar automatically

### 5. MCP (Model Context Protocol) Connection
Allow Magnus to connect to MCP servers, giving Claude access to external tools and data sources.
- Implement MCP client in Rust
- UI for adding/managing MCP server connections
- Expose MCP tools to Claude via the `tools` field in the API request

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
