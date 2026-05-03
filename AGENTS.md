# Magnus — Codex Desktop Alternative

## Working Mode

The developer is **learning Rust** through this project. Codex's role is pair-programming mentor for all Rust code:
- **Never write Rust code unless explicitly asked to**
- Explain concepts, guide implementation, review written code, and point out errors
- Ask the developer to attempt the code first, then give feedback
- React/TypeScript code can be written directly by Codex when asked



An open-source Codex desktop application built with Tauri (Rust + React) that supports configurable LLM providers, MCP tools, diagnostics, and future Agent Studio workflows.

## Project Goals

- Custom Provider support for external LLM gateways, self-hosted endpoints, and built-in providers
- Agent Studio for future Agent Workflows and Agent Loops
- Open-source alternative to Codex Desktop
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

### Rust Backend (backend/)
- All Provider API calls go through Rust using `reqwest`; the frontend never calls Provider APIs directly
- Tauri Commands (`#[tauri::command]`) are the public interface exposed to the frontend
- Keep Tauri Commands thin: they receive input, call a service, return output
- Business logic lives in modules, not in `main.rs` or `lib.rs` directly
- Use `thiserror` for typed errors, `anyhow` for internal error chaining

### React Frontend (frontend/)
- Frontend is purely a UI layer — it never calls Provider APIs directly
- Communicate with Rust via `@tauri-apps/api/core` `invoke()` calls
- Keep Tauri-specific code in a dedicated service/adapter layer (e.g., `frontend/services/tauri.ts`)
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
- Agents are future-only for now; current Magnus has Chats, not Agents
- Agent Studio is the future product area for Agents, Agent Workflows, and Agent Loops
- An Agent owns its own Provider, Model, and conversation context
- Agent Workflows are predefined steps; Agent Loops are deliberations until agents converge
- Agent Workflows and Agent Loops can be single-provider or multi-provider depending on their participating Agents

## Key Rust Crates to Know

| Crate | Purpose |
|-------|---------|
| `tauri` | Core framework |
| `reqwest` | HTTP client for Provider API calls |
| `serde` / `serde_json` | Serialization (Tauri Commands use this) |
| `tokio` | Async runtime |
| `thiserror` | Typed error definitions |
| `anyhow` | Error chaining for internal use |

## Key Tauri Concepts

- **Tauri Commands**: Rust functions exposed to the frontend via `invoke()`
- **Events**: Async messages from Rust to Frontend (good for streaming responses)
- **Permissions**: Declared in `tauri.conf.json` — follow least-privilege
- **App Data Dir**: Use `tauri::Manager::app_data_dir()` for persistent storage

## Provider Strategy

Provider identity and Provider Protocol are separate concepts.
A Provider is the configured source of models; a Provider Protocol is the API format Magnus uses to communicate with it.
Custom Providers cover external LLM gateways, self-hosted endpoints, and proxy-style URLs by storing a user-provided base URL plus Provider Protocol.
Do not introduce a separate Network Proxy concept until Magnus supports separate HTTP/HTTPS/SOCKS routing configuration.

```
Frontend (Provider Settings UI)
  -> invoke("upsert_provider", { provider, apiKey })
  -> Rust: save non-secret Provider Settings to App Data
  -> Rust: save Provider API Key to the System Secret Store
  -> Chat selects a Provider and Selected Model before sending
  -> Rust: route Provider API calls through the matching Provider Protocol client
```

Provider API Keys are secret material. They belong to one Provider and must stay in the System Secret Store, not in App Data or diagnostics exports.

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

## Domain Language and ADRs

- Keep ubiquitous language in `CONTEXT.md`
- Keep Architecture Decision Records in `docs/adr/`
- Before introducing or renaming product concepts, check `CONTEXT.md`
- Before changing a hard-to-reverse architecture rule, check existing ADRs and create a new one when the decision is surprising and trade-off driven

## Current State

### What's implemented
- **Settings**: Provider Settings and Default Provider saved to App Data
- **Provider API Keys**: stored separately in the System Secret Store
- **Providers**: Built-in Providers and Custom Providers with Provider Protocol support for Anthropic, OpenAI, and Google
- **Models**: selectable per next User Message; Model Messages record the Model that produced them
- **Chat**: Model Streaming Response creates Model Messages; Chat Transcript contains User Messages and Model Messages
- **Default Provider behavior**: New Chats start with the Default Provider; an empty Chat can switch Provider before the first User Message
- **Markdown rendering**: Model Messages render markdown with syntax-highlighted code blocks
- **Sidebar**: Left navbar with chat list, new chat button, right-click context menu (rename/delete), settings button at bottom
- **Multi-chat**: Multiple independent Chats, persisted to disk
- **Chat persistence**: Each chat saved as `{app_data_dir}/chats/{dd-mm-yy}-{uuid}.json`, loaded on startup
- **MCP Tools**: Connected MCP Servers expose MCP Tools globally to all Chats for the current runtime; Tool Call results belong to the invoking Chat and are not shared with other Chats by default
- **MCP persistence**: Configured MCP Servers persist in App Data; Connected MCP Server state is runtime-only and does not persist across app restarts
- **Diagnostics**: Diagnostic Events, Diagnostic Sessions, and Diagnostics Exports with redaction; Chat Transcripts excluded by default and only the Active Chat's Chat Transcript may be included by explicit user choice
- **Custom Provider example**: External Anthropic Proxy can be configured as a Custom Provider using the Anthropic Provider Protocol

### Known Issues / Limitations
- System Instruction support is future-only
- Agents are future-only; current Magnus has Chats, not Agents
- Agent Studio is only a working direction for now
- Chat has one Provider, but the persisted Chat-level Model behavior is temporary and not settled
- Blocking existing Chats after their Provider is deleted is intended but not implemented yet
- Current Chat Transcripts do not include Tool Calls or Tool Call results
- Diagnostics Retention currently prunes old diagnostic event files after 7 days and caps retained diagnostic files at about 5 MiB; treat these numbers as current behavior, not a permanent product promise

### Custom Provider notes
An external LLM proxy URL such as `https://llm-proxy.edgez.live/` is a Custom Provider, not a separate proxy concept. It can use the Anthropic Provider Protocol, so requests follow the Anthropic-compatible URL shape such as `{base_url}v1/messages` when the base URL already has a trailing slash.

## Current File Structure

```
frontend/
├── App.tsx                     # Routing — keeps HomePage always mounted to preserve state
├── components/
│   ├── HomePage.tsx            # State orchestration for Chats
│   ├── Sidebar.tsx             # Chat list, new chat, rename, settings button
│   ├── ChatArea.tsx            # Messages display + input
│   └── SettingsPage.tsx        # Provider Settings + Provider API Key form

backend/src/
├── main.rs                     # Entry point — calls lib::run()
├── lib.rs                      # Tauri command registration
├── config.rs                   # Settings and Provider Settings, save/load to App Data
├── secrets.rs                  # Provider API Keys in the System Secret Store
├── llm/                        # Provider Protocol clients
├── mcp/                        # MCP Server connections and MCP Tool execution
├── diagnostics/                # Diagnostics, Diagnostic Events, exports, redaction
└── chats.rs                    # Chat struct, save/load/delete per-file
```

## Future Features

```
magnus/
├── frontend/                   # React frontend
│   ├── components/             # UI components
│   ├── services/               # Tauri invoke wrappers
│   ├── stores/                 # State management
│   ├── App.tsx
│   └── main.tsx
├── backend/                  # Rust backend
│   ├── src/
│   │   ├── main.rs             # Tauri app setup only
│   │   ├── lib.rs              # Command registration
│   │   ├── llm/                # Provider Protocol clients
│   │   ├── mcp/                # MCP Server and MCP Tool support
│   │   ├── diagnostics/        # Diagnostics capture and exports
│   │   ├── agents/             # Agent management
│   │   └── config/             # Settings and Provider Settings
│   ├── Cargo.toml
│   └── tauri.conf.json
├── AGENTS.md
├── package.json
└── vite.config.ts
```

The following features are planned but not yet implemented:

### 1. Agent Studio
Create a separate product area for multi-agent functionality.
- Agents own Provider, Model, and conversation context
- Agent Workflows are predefined steps toward a goal
- Agent Loops are deliberations where Agents discuss approaches until convergence
- Agent Studio should not be bolted onto the normal Chat screen

### 2. System Instructions
Add non-user instructions that shape how a Model or Agent behaves.
- Use the term System Instruction, not system prompt
- System Instructions are not User Messages

### 3. Computer Task Manipulation / Tooling
Allow Codex to interact with the local filesystem (move, copy, create, delete files).
- Expose Tauri Commands for filesystem operations with permission prompts
- Connect to Codex's tool_use API feature
- Security: strict permission model, user must approve each action type

### 4. MCP Auto-Connect
Optionally reconnect Configured MCP Servers when Magnus starts.
- Current behavior persists Configured MCP Servers only
- Connected MCP Server state is runtime-only
- Auto-connect needs an explicit product and security decision before implementation

### 5. Blocked Chats for Deleted Providers
Keep existing Chats visible when their Provider is deleted, but prevent new User Messages until multi-provider Chat behavior is designed.
- Current code may leave old Chats pointing at a missing Provider
- Intended behavior is a visible Blocked Chat, not silent deletion or automatic Provider reassignment

## Conventions

- Rust: follow `rustfmt` defaults, use `clippy` warnings as errors in CI
- TypeScript: strict mode enabled
- Name Tauri Commands in `snake_case` (Rust) — they map to camelCase in the frontend by convention
- Errors returned from Tauri Commands should be `String` (serializable) at the command boundary

## Learning Resources

- Tauri v2 docs: https://v2.tauri.app
- Tauri commands guide: https://v2.tauri.app/develop/calling-rust/
- reqwest docs: https://docs.rs/reqwest/latest/reqwest/
- Tauri state management: https://v2.tauri.app/develop/state-management/
