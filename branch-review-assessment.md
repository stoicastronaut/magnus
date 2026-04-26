# Branch Review Assessment

Branch reviewed: `feat/add-more-providers`

Comparison base: `main`

Scope: provider management, multi-provider chat flow, MCP integration, frontend settings/chat updates, backend LLM/provider plumbing.

## Findings

### 1. High: MCP tools are auto-executed without any user approval step

Files:
- `backend/src/lib.rs:217`
- `backend/src/lib.rs:268`

Why it matters:
- Every connected MCP tool is exposed to every chat request.
- When the model emits a tool call, the backend executes it immediately and returns the result to the model.
- There is no allowlist, confirmation dialog, per-tool consent, or distinction between low-risk and high-risk tools.
- In practice this means prompt injection or a bad model decision can trigger filesystem, network, or external-service actions as soon as the user has connected a capable MCP server.

Assessment:
- This is the biggest security concern in the branch.
- The project notes mention a future permission model for computer task manipulation, but the current implementation already creates an approval-free execution path for MCP-backed tools.

Recommendation:
- Add a user approval gate before executing tool calls.
- At minimum, support deny/allow-once/allow-for-session behavior and show the tool name plus arguments.
- Consider server- and tool-level allowlists so only explicitly trusted tools can run automatically.

### 2. High: MCP secrets are stored in plaintext on disk

Files:
- `backend/src/mcp/mod.rs:9`
- `backend/src/mcp/mod.rs:31`
- `backend/src/mcp/mod.rs:43`

Why it matters:
- `McpServer` includes `token` and `env_key`, and `save_servers` serializes the whole struct into `mcp_servers.json`.
- That places MCP access tokens in cleartext in the app data directory.
- Anyone or any process with access to the local profile can recover those credentials.

Assessment:
- This is inconsistent with the stronger handling used for provider API keys, which go through the OS keychain in `backend/src/secrets.rs`.
- It is especially risky because MCP tokens are often GitHub, Linear, or other high-value personal access tokens.

Recommendation:
- Move MCP tokens into the keychain using the same pattern as provider API keys.
- Persist only stable metadata in JSON, plus a keychain lookup key.
- If plaintext storage is temporarily unavoidable, the UI should clearly warn users before saving.

### 3. Medium: the branch leaves backend unit tests broken

Files:
- `backend/src/config.rs:66`
- `backend/src/chats.rs:49`

Validated by:
- `cargo test --manifest-path backend/Cargo.toml`

Observed result:
- The test target does not compile because the tests still reference removed fields such as `Settings.api_key`, `Settings.base_url`, and omit the new `Chat.provider_id` / `Message.model_id` fields.
- There is also an old filename assertion in `backend/src/chats.rs:90` that no longer matches the saved filename shape.

Why it matters:
- This is more than missing coverage: the branch actively breaks the backend test suite.
- It reduces confidence in later refactors around provider persistence and chat serialization because the safety net is already disabled.

Recommendation:
- Update the `config.rs` and `chats.rs` tests to the new provider-based schema before merging.
- After that, add coverage for provider round-tripping, backward compatibility for old chats, and model/provider metadata persistence.

### 4. Medium: debug logging leaks chat content and provider metadata

Files:
- `backend/src/lib.rs:198`
- `backend/src/lib.rs:211`
- `backend/src/lib.rs:213`
- `backend/src/llm/anthropic.rs:181`
- `backend/src/llm/anthropic.rs:195`
- `backend/src/llm/anthropic.rs:228`

Why it matters:
- `stream_message` logs provider identifiers, the full provider debug representation, and API key length.
- `anthropic.stream_raw` logs provider responses, raw error bodies, chunk counts, and every emitted token.
- Token-by-token logging means model output, and potentially sensitive user content echoed by the model, is written to logs.

Assessment:
- This may be acceptable during local debugging, but it is too verbose for a default code path in an app that handles API keys, prompts, and tool results.
- The main risk here is privacy leakage rather than remote compromise.

Recommendation:
- Remove the logs or gate them behind a debug feature flag / environment variable.
- Never log full streamed content or raw provider error bodies in production builds.

### 5. Medium: model selection state is global, can drift from the active provider, and may send invalid model IDs

Files:
- `frontend/components/HomePage.tsx:38`
- `frontend/components/HomePage.tsx:128`
- `frontend/components/ModelPicker.tsx:17`
- `frontend/components/ModelPicker.tsx:44`
- `frontend/components/ChatArea.tsx:137`

Why it matters:
- `selectedModelId` lives once at the page level rather than per chat or per provider.
- When the active chat changes, the code does not reconcile that state with the newly active provider.
- `ModelPicker` only auto-selects a model when `value` is falsy, so switching from an OpenAI chat to a Google chat can leave a stale model ID such as `gpt-5` in state.
- `ChatArea` treats any truthy `selectedModelId` as sendable, and `handleSend` forwards it directly to the backend.

Assessment:
- This is primarily a correctness and clean-state-management issue.
- I am inferring the runtime behavior from the state flow rather than from an interactive repro, but the code path is clear enough to be a real concern.

Recommendation:
- Store the selected model per chat, or derive it from the chat’s last assistant message / provider.
- When the effective provider changes, validate the current model against that provider’s available models and reset it if it is no longer valid.

### 6. Low: custom provider base URLs are not normalized before endpoint concatenation

Files:
- `frontend/components/ProviderEditModal.tsx:45`
- `frontend/components/ProviderEditModal.tsx:53`
- `backend/src/llm/anthropic.rs:151`
- `backend/src/llm/openai.rs:49`
- `backend/src/llm/gemini.rs:58`

Why it matters:
- Custom base URLs are trimmed but not normalized.
- The backend concatenates endpoint paths directly with string formatting, assuming the base URL already ends with a trailing slash.
- A user-entered value like `https://proxy.example.com/api` becomes malformed when the code produces `https://proxy.example.com/apichat/completions` or `...apiv1/messages`.

Assessment:
- This is more of a reliability/clean-code concern than a security issue.
- It will be an easy source of hard-to-diagnose provider setup failures.

Recommendation:
- Normalize base URLs on save or in the backend client constructors.
- Prefer URL joining over raw string concatenation.

## What Looks Good

- The frontend keeps Tauri IPC calls in `frontend/services/tauri.ts`, which is a good boundary and makes the UI easier to reason about.
- Provider modeling is cleaner than the previous single-provider shape. The `ProviderType` enum and `models_for_provider` split are straightforward to follow.
- Provider API keys are moved out of JSON settings and into the OS keychain, which is the right direction.
- Markdown rendering uses `react-markdown` without enabling raw HTML, which is a good default from a frontend security perspective.
- Frontend tests still pass with `pnpm test`, so the React-side refactor at least retained its existing test baseline.

## Validation Performed

- `pnpm lint`
- `pnpm test`
- `cargo fmt --check --manifest-path backend/Cargo.toml`
- `cargo test --manifest-path backend/Cargo.toml`

Results:
- `pnpm lint`: passed
- `pnpm test`: passed
- `cargo fmt --check --manifest-path backend/Cargo.toml`: passed
- `cargo test --manifest-path backend/Cargo.toml`: failed because of stale backend tests

## Overall Assessment

This branch moves the architecture in a useful direction: typed provider definitions, a dedicated Tauri service layer, multi-provider UI, and stronger provider-key handling are all solid improvements. The main blockers are security hardening around MCP tool execution and secret storage, plus the fact that the backend test suite is currently broken. I would not merge the branch without addressing those items first.
