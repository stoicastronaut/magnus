# Multi-Provider & Multi-Model Support

## Context

Magnus today hardcodes Anthropic as the only LLM provider and `claude-haiku-4-5-20251001` as the only model. `Settings` holds a single `api_key` + `base_url`, and `stream_message` unconditionally builds an Anthropic-shaped request.

We want users to:

1. Configure multiple providers in Settings — three built-ins (Anthropic, OpenAI, Gemini) plus unlimited **custom** providers that speak either the Anthropic or OpenAI wire protocol (the current `edgez.live` proxy is a custom-Anthropic provider).
2. Store each provider's API key in the OS keychain (not plaintext on disk).
3. Pick a **default provider** globally; new chats inherit it.
4. Pick a provider at chat creation; it **locks** once the first message is sent.
5. Pick a **model per message** from the active provider's model list. Each assistant message remembers which model produced it.

Models are hardcoded per provider in Rust — adding new models is a code change, not a config change.

---

## Decisions (locked)

| Decision | Choice |
|---|---|
| Model list source | Hardcoded in Rust |
| Custom-provider protocol | User picks Anthropic or OpenAI at creation |
| Provider scope | Per-chat, locked after first message |
| Model scope | Per-message (assistant messages store `model_id`) |
| API key storage | OS keychain (`keyring` crate) |
| Built-in scope (this PR) | All three: Anthropic, OpenAI, Gemini |

---

## Backend (Rust) — you write this

### 1. New crate dependency

Add to `backend/Cargo.toml`:

```toml
keyring = "3"
```

### 2. Data model changes

#### 2a. `backend/src/config.rs` — rewrite `Settings`

```rust
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Settings {
    pub default_provider_id: Option<String>,
    pub providers: Vec<ProviderConfig>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ProviderConfig {
    pub id: String,                // "anthropic" | "openai" | "gemini" | uuid for custom
    pub display_name: String,      // "Anthropic", "OpenAI", "Gemini", or user-entered for custom
    #[serde(flatten)]
    pub kind: ProviderKind,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ProviderKind {
    BuiltIn { which: BuiltInId },
    Custom { protocol: Protocol, base_url: String },
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BuiltInId { Anthropic, OpenAI, Gemini }

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Protocol { Anthropic, OpenAI }
// Gemini is NOT selectable for Custom (user said "Anthropic or OpenAI").
```

Note the **API key is NOT on `ProviderConfig`** — it lives in the keychain, keyed by `provider.id`.

#### 2b. `backend/src/chats.rs` — extend `Chat` and `Message`

```rust
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Message {
    pub role: String,
    pub content: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model_id: Option<String>,   // Some(..) on assistant messages; None on user messages
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Chat {
    pub id: String,
    pub name: String,
    pub messages: Vec<Message>,
    pub created_at: String,
    pub provider_id: String,        // the id of the provider locked in for this chat
}
```

#### 2c. Migration

Old files won't have `provider_id` / `model_id`. Options:

- `Message.model_id`: `#[serde(default)]` → reads as `None` from old files. Done.
- `Chat.provider_id`: can't default cleanly. In `load_chats`, if the field is missing, inject `settings.default_provider_id` (or create an "anthropic" built-in if no default).
- Old `Settings { api_key, base_url }`: in `Settings::load`, detect the old shape (try old struct first on parse error). If found:
  - If `base_url == "https://api.anthropic.com"` (or empty): create one `BuiltIn { Anthropic }` provider, write `api_key` to keychain under `"anthropic"`, set it as default.
  - Else: create a `Custom { protocol: Anthropic, base_url }` with id = new uuid, display_name `"Legacy"`, same treatment.
  - Save the new shape back to disk.

### 3. Models module — new file `backend/src/models.rs`

```rust
use crate::config::BuiltInId;

#[derive(Clone, Debug, serde::Serialize)]
pub struct ModelInfo {
    pub id: String,          // API model string
    pub display_name: String,
}

pub fn models_for(which: BuiltInId) -> Vec<ModelInfo> {
    match which {
        BuiltInId::Anthropic => vec![
            ModelInfo { id: "claude-haiku-4-5-20251001".into(),  display_name: "Haiku 4.5".into() },
            ModelInfo { id: "claude-sonnet-4-6".into(),          display_name: "Sonnet 4.6".into() },
            ModelInfo { id: "claude-opus-4-7".into(),            display_name: "Opus 4.7".into() },
        ],
        BuiltInId::OpenAI => vec![
            ModelInfo { id: "gpt-5".into(),            display_name: "GPT-5".into() },
            ModelInfo { id: "gpt-5-mini".into(),       display_name: "GPT-5 mini".into() },
            ModelInfo { id: "gpt-4o".into(),           display_name: "GPT-4o".into() },
        ],
        BuiltInId::Gemini => vec![
            ModelInfo { id: "gemini-2.5-pro".into(),   display_name: "Gemini 2.5 Pro".into() },
            ModelInfo { id: "gemini-2.5-flash".into(), display_name: "Gemini 2.5 Flash".into() },
        ],
    }
}
```

> Double-check each id string against each provider's current API — this table is the thing that'll go stale.

For a **custom** provider, return the model list for the protocol it speaks (Anthropic-protocol custom → Anthropic models; OpenAI-protocol custom → OpenAI models). Centralise in one function:

```rust
pub fn models_for_provider(p: &ProviderConfig) -> Vec<ModelInfo> {
    match &p.kind {
        ProviderKind::BuiltIn { which } => models_for(*which),
        ProviderKind::Custom { protocol, .. } => match protocol {
            Protocol::Anthropic => models_for(BuiltInId::Anthropic),
            Protocol::OpenAI    => models_for(BuiltInId::OpenAI),
        },
    }
}
```

### 4. Provider clients — `backend/src/llm/` directory

Split the current `llm.rs` into a directory:

```
backend/src/llm/
├── mod.rs          # enum + dispatch
├── anthropic.rs    # existing code, refactored into an impl
├── openai.rs       # new
└── gemini.rs       # new
```

#### 4a. The enum (`llm/mod.rs`)

Instead of a trait with dynamic dispatch, use an enum with a variant per client. All providers are known at compile time so this avoids heap allocation and `async-trait`.

```rust
pub enum LlmClient {
    Anthropic(AnthropicClient),
    OpenAI(OpenAIClient),
    Gemini(GeminiClient),
}

impl LlmClient {
    pub async fn stream(
        &self,
        app: &tauri::AppHandle,
        messages: &[crate::chats::Message],
        model_id: &str,
    ) -> Result<String, String> {
        match self {
            LlmClient::Anthropic(c) => c.stream(app, messages, model_id).await,
            LlmClient::OpenAI(c)    => c.stream(app, messages, model_id).await,
            LlmClient::Gemini(c)    => c.stream(app, messages, model_id).await,
        }
    }

    pub async fn generate_title(
        &self,
        messages: &[crate::chats::Message],
        model_id: &str,
    ) -> Result<String, String> {
        match self {
            LlmClient::Anthropic(c) => c.generate_title(messages, model_id).await,
            LlmClient::OpenAI(c)    => c.generate_title(messages, model_id).await,
            LlmClient::Gemini(c)    => c.generate_title(messages, model_id).await,
        }
    }
}
```

The enum emits `stream-token` events (same event name as today) so the frontend streaming UI needs zero changes to the event channel.

#### 4b. Dispatch

```rust
pub fn client_for(
    provider: &ProviderConfig,
    api_key: String,
    http: reqwest::Client,
) -> LlmClient {
    match &provider.kind {
        ProviderKind::BuiltIn { which: BuiltInId::Anthropic } => LlmClient::Anthropic(AnthropicClient::new("https://api.anthropic.com/".into(), api_key, http)),
        ProviderKind::BuiltIn { which: BuiltInId::OpenAI }    => LlmClient::OpenAI(OpenAIClient::new("https://api.openai.com/v1/".into(), api_key, http)),
        ProviderKind::BuiltIn { which: BuiltInId::Gemini }    => LlmClient::Gemini(GeminiClient::new("https://generativelanguage.googleapis.com/v1beta/".into(), api_key, http)),
        ProviderKind::Custom { protocol: Protocol::Anthropic, base_url } => LlmClient::Anthropic(AnthropicClient::new(base_url.clone(), api_key, http)),
        ProviderKind::Custom { protocol: Protocol::OpenAI, base_url }    => LlmClient::OpenAI(OpenAIClient::new(base_url.clone(), api_key, http)),
    }
}
```

#### 4c. Anthropic client (`llm/anthropic.rs`)

Move the existing `stream_message` body here, untouched except:
- Take `base_url` + `api_key` from constructor, not from command args.
- Take `model_id` parameter instead of the hardcoded string on the line currently at `llm.rs:143`.

Wire format: same as today (`POST {base_url}v1/messages` with `x-api-key` header, `stream: true`, SSE `content_block_delta` / `text_delta` events).

#### 4d. OpenAI client (`llm/openai.rs`)

`POST {base_url}chat/completions` (note: built-in base_url already ends `/v1/`).

Request shape:
```json
{
  "model": "<model_id>",
  "stream": true,
  "messages": [{"role": "user", "content": "..."}, ...]
}
```

Headers: `Authorization: Bearer <api_key>`, `Content-Type: application/json`.

SSE parsing: lines prefixed `data: `. Payload per chunk:
```json
{"choices":[{"delta":{"content":"piece of text"}, "finish_reason": null}]}
```
Emit `app.emit("stream-token", delta.content)` for each non-empty `delta.content`. Terminator is `data: [DONE]`.

Assemble the full text as you go; return it at the end.

#### 4e. Gemini client (`llm/gemini.rs`)

Gemini's streaming endpoint: `POST {base_url}models/{model_id}:streamGenerateContent?alt=sse&key={api_key}`.

Request body:
```json
{
  "contents": [
    {"role": "user",  "parts": [{"text": "..."}]},
    {"role": "model", "parts": [{"text": "..."}]}
  ]
}
```

Note Gemini uses `"model"` where Anthropic/OpenAI use `"assistant"` — translate.

SSE payload per chunk:
```json
{"candidates":[{"content":{"parts":[{"text":"piece"}], "role":"model"}}]}
```
Emit each `parts[0].text` as `stream-token`.

### 5. Keychain access — `backend/src/secrets.rs` (new)

Thin wrapper so command code doesn't touch `keyring` directly:

```rust
const SERVICE: &str = "com.stoicastronaut.magnus";

pub fn set_api_key(provider_id: &str, key: &str) -> Result<(), String> {
    keyring::Entry::new(SERVICE, provider_id).and_then(|e| e.set_password(key))
        .map_err(|e| e.to_string())
}
pub fn get_api_key(provider_id: &str) -> Result<String, String> { ... }
pub fn delete_api_key(provider_id: &str) -> Result<(), String> { ... }
```

### 6. Tauri commands — `backend/src/lib.rs` changes

Remove: nothing (keep `save_settings` / `get_settings` but change their shape).

Change:
- `stream_message(app, pool, chat_id, provider_id, model_id, messages)` — look up provider from settings, fetch key from keychain, dispatch via `client_for(..).stream(..)`.
- `rename_chat(app, provider_id, chat)` — same lookup; pick a cheap model for that provider (e.g. Haiku / gpt-5-mini / Flash). Add a helper `cheapest_model(provider)` in `models.rs`.

Add:
- `upsert_provider(app, provider: ProviderConfig, api_key: Option<String>)` — saves config; if `api_key` is `Some`, writes it to keychain. (Omit the key when the user is only editing display_name/base_url.)
- `delete_provider(app, provider_id: String)` — removes from settings; deletes key from keychain; refuses if `provider_id == default_provider_id` and there are other providers (force user to pick a new default first).
- `set_default_provider(app, provider_id: String)`.
- `list_models(app, provider_id: String) -> Vec<ModelInfo>` — looks up the provider, delegates to `models_for_provider`.
- `has_api_key(provider_id: String) -> bool` — so the UI can show a "configured" badge without reading the key.

All commands return `Result<T, String>` per your convention.

### 7. File/module layout after changes

```
backend/src/
├── main.rs          (unchanged)
├── lib.rs           (command registration — updated)
├── config.rs        (Settings / ProviderConfig / ProviderKind / BuiltInId / Protocol; migration)
├── chats.rs         (Message.model_id, Chat.provider_id)
├── models.rs        NEW
├── secrets.rs       NEW
├── llm/
│   ├── mod.rs       (trait + dispatch)
│   ├── anthropic.rs (extracted from today's llm.rs)
│   ├── openai.rs    NEW
│   └── gemini.rs    NEW
└── mcp/             (unchanged)
```

---

## Frontend (TypeScript/React) — I write this

### 1. New service layer: `frontend/services/tauri.ts`

Consolidate all `invoke()` calls behind typed functions. Today they're scattered across HomePage/SettingsPage. This is cheap to do now while we're touching the surface area and makes the rest of the plan readable.

Exports: `getSettings`, `saveProvider`, `deleteProvider`, `setDefaultProvider`, `listModels`, `hasApiKey`, `streamMessage`, `saveChat`, `renameChat`, `loadChats`, `deleteChat`.

Also export shared TS types mirroring the new Rust shapes: `ProviderConfig`, `ProviderKind`, `BuiltInId`, `Protocol`, `ModelInfo`, `Chat`, `Message` (with optional `model_id`).

### 2. Settings page rewrite — `frontend/components/SettingsPage.tsx`

Replace the current API Configuration section (MCP section stays as-is). New layout, top to bottom:

- **"Default provider"** — dropdown of configured providers. Disabled when zero are configured.
- **"Your providers"** — list of cards (one row each) for already-configured providers. Each row: icon, display_name, small tag (`Built-in` / `Custom · Anthropic` / `Custom · OpenAI`), edit button, delete button.
- **"Add a provider"** — 4 tiles:
  - Anthropic (icon)
  - OpenAI (icon)
  - Gemini (icon)
  - "+ Custom"

  Tiles for already-configured built-ins appear disabled/greyed (each built-in only configurable once).

Clicking a tile or Edit opens a modal (see §3).

### 3. New component: `components/ProviderEditModal.tsx`

Two modes based on props: `builtIn: BuiltInId` **or** `custom: true`.

- **Built-in mode**: read-only name + icon; one field — API key (password input). On save, `saveProvider({ id: <builtInId>, display_name, kind: { kind: "built_in", which } }, apiKey)`.
- **Custom mode**: display name (required), protocol radio (Anthropic / OpenAI), base URL (required, validated as URL), API key. On save, `saveProvider({ id: <existing or crypto.randomUUID()>, display_name, kind: { kind: "custom", protocol, base_url } }, apiKey)`.
- Editing an existing provider: fields prefilled; API key field left blank with placeholder "•••••• (leave blank to keep current)". Pass `apiKey` only if non-empty.

### 4. `components/HomePage.tsx` changes

Add state: `settings: Settings` (fetched on mount so we know the default provider).

When creating a new chat:
```ts
{
  id, name, created_at,
  messages: [],
  provider_id: settings.default_provider_id,
}
```

When calling `streamMessage`, pass `chat.provider_id` and the currently-selected `modelId` from ChatArea. When the response returns, append the assistant message with `model_id: modelId`.

### 5. `components/ChatArea.tsx` changes

New props: `providerId`, `providers`, `onProviderChange(id)`, `messages` (unchanged shape but now `model_id` is readable).

Add two new pieces of UI:

- **Header bar** (top of chat): shows the active provider.
  - If `messages.length === 0` → render as `<ProviderPicker>` (dropdown).
  - If `messages.length > 0` → render as static label with a small lock icon and tooltip "Provider locked after first message."
- **Model picker** inside the input footer (left of the send button): `<ModelPicker providerId={...} value={modelId} onChange={...} />`. Default value: first model from `listModels(providerId)`.

Optional polish: on assistant messages where `model_id` is set, render a faint tag below the content (e.g. `Opus 4.7`).

### 6. New components

- `components/ProviderPicker.tsx` — dropdown of configured providers for selecting a new chat's provider.
- `components/ModelPicker.tsx` — fetches models via `listModels(providerId)` on mount and whenever `providerId` changes. Small dropdown; shows `display_name`.
- `components/ProviderIcon.tsx` — renders the right icon/emoji for a provider (Anthropic / OpenAI / Gemini / generic for custom). SVGs inline; no new asset pipeline.

### 7. Empty-state handling

If the user has **no providers configured**, HomePage disables chat creation and shows a prompt to open Settings. Today the equivalent check is `hasSettings`; replace that with `settings.providers.length > 0 && settings.default_provider_id != null`.

---

## Critical files

Modified:
- `backend/src/config.rs`
- `backend/src/chats.rs`
- `backend/src/lib.rs`
- `backend/Cargo.toml`
- `frontend/components/HomePage.tsx`
- `frontend/components/ChatArea.tsx`
- `frontend/components/SettingsPage.tsx`
- `frontend/components/Sidebar.tsx` (update shared `Chat`/`Message`/`Settings` type exports)

Created:
- `backend/src/models.rs`
- `backend/src/secrets.rs`
- `backend/src/llm/mod.rs`
- `backend/src/llm/anthropic.rs` (extract from current `llm.rs`)
- `backend/src/llm/openai.rs`
- `backend/src/llm/gemini.rs`
- `frontend/services/tauri.ts`
- `frontend/components/ProviderEditModal.tsx`
- `frontend/components/ProviderPicker.tsx`
- `frontend/components/ModelPicker.tsx`
- `frontend/components/ProviderIcon.tsx`

Deleted:
- `backend/src/llm.rs` (contents moved into `llm/anthropic.rs`)

---

## Verification

1. `cargo tauri dev` boots. App loads and migrates the existing `settings.json` into the new shape without prompting.
2. In Settings, the pre-existing Anthropic key now shows up as a configured built-in (or Custom/Anthropic if the user was on the `edgez.live` proxy).
3. Add an OpenAI key → `OpenAI` tile turns configured → new chat → set default to OpenAI → new chat picks OpenAI → send "hello" → tokens stream → assistant message saved with `model_id: "gpt-5"`.
4. Repeat for Gemini.
5. Add a custom provider speaking Anthropic protocol pointing to `https://llm-proxy.edgez.live/` → send message → works.
6. Create a chat, send one message, confirm the provider picker switches to a locked label.
7. In one chat, switch the model between messages (Haiku → Opus → Haiku). Inspect the persisted chat JSON and confirm each assistant message has the correct `model_id`.
8. Delete a provider → its key disappears from the OS keychain (verify via Keychain Access on macOS).
9. Attempt to delete the default provider while others exist → command refuses with an error.
10. Restart the app → everything reloads: default provider, providers list, chats with their locked provider, per-message model badges.
