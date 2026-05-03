# Magnus

Magnus is a lightweight desktop application for running AI chats through locally controlled providers, tools, and diagnostics. This context captures the domain language used across the product.

## Language

### Chats

**Chat**:
A persisted user conversation with its own messages and provider selection.
_Avoid_: Session, thread, conversation

**Chat Name**:
The user-visible label for a Chat.
_Avoid_: Title, generated title

**Active Chat**:
The Chat currently selected by the user.
_Avoid_: Current session, selected thread

**Chat Transcript**:
The ordered User Messages and Model Messages in a Chat.
_Avoid_: Transcript, history

**User Message**:
A message in a Chat written by the user.
_Avoid_: Prompt

**Model Message**:
A message in a Chat produced by the selected Model.
_Avoid_: Assistant message, completion

**Model Streaming Response**:
The live delivery of a Model Message as chunks arrive.
_Avoid_: Stream, completion

**System Instruction**:
A future non-user instruction that defines how a Model or Agent should behave.
_Avoid_: System prompt, prompt, context

### Providers

**Provider**:
A configured source of LLM models, either built in or custom.
_Avoid_: Connection, proxy

**Provider Settings**:
The non-secret configuration for one Provider.
_Avoid_: Provider configuration, connection settings

**Default Provider**:
The Provider used when creating a new Chat unless the user chooses another Provider.
_Avoid_: Built-in provider

**Blocked Chat**:
A Chat that can be viewed but cannot accept new User Messages because its Provider is unavailable.
_Avoid_: Disabled chat, broken chat

**Built-in Provider**:
A Provider whose protocol and base URL are supplied by Magnus.
_Avoid_: Default provider, official provider

**Custom Provider**:
A Provider whose protocol and base URL are configured by the user.
_Avoid_: Proxy, endpoint

**Provider Protocol**:
The API format Magnus uses to communicate with a Provider.
_Avoid_: Protocol, API format, message format

**Model**:
A selectable LLM offered by a Provider.
_Avoid_: Engine

**Selected Model**:
The Model chosen in the UI for the next User Message.
_Avoid_: Chat model, active model

**Provider API Key**:
A secret credential associated with one Provider.
_Avoid_: API key, token

**Agent**:
A future autonomous participant that may use its own Provider, Model, and conversation context.
_Avoid_: Chat, session

**Agent Studio**:
A future product area for configuring and running Agents, Agent Workflows, and Agent Loops separately from normal Chats.
_Avoid_: Agent flow, agent workspace, agent mode

**Agent Workflow**:
A predefined set of steps performed by one or more Agents, possibly using different Providers, to achieve a goal.
_Avoid_: Loop, discussion

**Agent Loop**:
A deliberation between multiple Agents, possibly using different Providers, that continues until they converge on an implementation, review, test, or other decision.
_Avoid_: Workflow, LLM loop, debate

### Diagnostics

**Diagnostics**:
The product area for troubleshooting information captured by Magnus.
_Avoid_: Logs

**Diagnostic Event**:
One recorded troubleshooting event from the frontend or backend.
_Avoid_: Log line, log entry

**Diagnostic Session**:
One runtime launch of Magnus used to correlate logs, crashes, dropped diagnostic events, and diagnostics exports.
_Avoid_: Session

**Diagnostics Export**:
A user-created troubleshooting bundle or summary with redacted diagnostic information.
_Avoid_: Log export, dump

**Diagnostics Retention**:
The current pruning behavior for local diagnostic information.
_Avoid_: Log retention

### Persistence

**App Data**:
Magnus-owned local data stored in the operating system's application data directory.
_Avoid_: Local state, storage

**System Secret Store**:
The operating system credential store used for Provider API Keys.
_Avoid_: App Data, settings file

**Settings**:
User-managed app configuration stored in App Data.
_Avoid_: Preferences, configuration

### Tools

**Tauri Command**:
A frontend-to-backend operation exposed by Magnus through Tauri.
_Avoid_: Command

**MCP Server**:
A configured external process that exposes MCP Tools to Magnus.
_Avoid_: Tool provider, capability provider

**Configured MCP Server**:
An MCP Server definition saved in App Data with the launch details Magnus needs to start or reconnect it.
_Avoid_: Saved MCP connection

**Connected MCP Server**:
A Configured MCP Server with an active runtime connection available for listing MCP Tools and executing Tool Calls.
_Avoid_: Configured MCP Server, saved server

**MCP Tool**:
A callable capability exposed by an MCP Server.
_Avoid_: Capability, command

**Tool Call**:
One request from a Model or Agent to execute an MCP Tool.
_Avoid_: Command, action

## Relationships

- A **Chat** can produce diagnostic events during a **Diagnostic Session**
- A **Chat** has one **Chat Name**
- A **Chat** contains **User Messages** and **Model Messages**
- A **Chat Transcript** belongs to exactly one **Chat**
- There is at most one **Active Chat** at a time
- A **Model Streaming Response** produces one **Model Message**
- A **Selected Model** is used for the next **User Message**
- A **Model Message** records the **Model** that produced it
- A **Chat** has one **Provider**, but does not currently have a persisted Chat-level Model
- A future **System Instruction** may shape a **Model** or **Agent**, but is not a **User Message**
- A **Diagnostic Session** can contain events from zero or more **Chats**
- **Diagnostics** contain **Diagnostic Events**
- **Diagnostics Retention** currently prunes old diagnostic event files after 7 days and caps retained diagnostic files at about 5 MiB
- A **Diagnostics Export** includes redacted diagnostic information from one or more **Diagnostic Sessions**
- A **Diagnostics Export** excludes Chat transcripts by default
- A **Diagnostics Export** may include the **Active Chat's** **Chat Transcript** only when the user explicitly chooses that option
- A failed **Tool Call** can produce a **Diagnostic Event**
- **Diagnostic Events** may reference Chat, Provider, Model, MCP Server, or MCP Tool identifiers when available
- **App Data** contains Settings, Chats, diagnostics, and MCP Server configuration
- **Settings** contain **Provider Settings** and the default Provider selection
- A **Provider API Key** belongs to exactly one **Provider**
- A **Provider API Key** is secret material and is not normal **Provider Settings**
- A **Provider API Key** is stored in the **System Secret Store**, not in **App Data**
- A **Provider** offers one or more **Models**
- The **Default Provider** can be a **Built-in Provider** or a **Custom Provider**
- A **Custom Provider** selects one **Provider Protocol**
- A new **Chat** starts with the **Default Provider**
- A **Chat** uses exactly one **Provider**
- An empty **Chat** can switch Provider before the first **User Message**
- A **Chat's** Provider is selected before the first message and cannot be changed once messages exist
- A **Chat** whose **Provider** has been deleted should become a **Blocked Chat** until multi-provider Chat behavior is designed
- A **Custom Provider** can represent an external Anthropic proxy, a self-hosted LLM gateway, or another user-configured endpoint
- Current Magnus has **Chats**, not **Agents**
- A future **Agent** may use a different **Provider** from other Agents in the same **Agent Workflow** or **Agent Loop**
- **Agent Studio** contains future **Agent Workflows** and **Agent Loops**
- Provider diversity belongs to the participating **Agents**, not to the **Agent Workflow** or **Agent Loop** itself
- An **Agent Workflow** or **Agent Loop** can be single-provider or multi-provider depending on the participating **Agents**
- An **Agent Workflow** is step-defined; an **Agent Loop** is deliberation-defined
- A **Configured MCP Server** can become a **Connected MCP Server**
- A **Connected MCP Server** exposes zero or more **MCP Tools**
- **Configured MCP Servers** persist in **App Data**
- **Connected MCP Server** state is runtime-only and does not persist across app restarts
- A **Tauri Command** is called by the frontend; an **MCP Tool** is called by a Model or Agent
- A **Tool Call** targets exactly one **MCP Tool**
- A **Tool Call** may be produced while handling a **Chat** message or future **Agent** activity
- A **Connected MCP Server** makes its **MCP Tools** globally available to all **Chats** during the current runtime
- **Tool Call** results belong to the **Chat** or future **Agent** activity that invoked them and are not shared with other **Chats** by default
- Current **Chat Transcripts** do not include **Tool Calls** or Tool Call results

## Example dialogue

> **Dev:** "Did this error happen in the **Chat** or in the **Diagnostic Session**?"
> **Domain expert:** "The failed message belongs to the **Chat**; the log correlation ID belongs to the **Diagnostic Session**."
>
> **Dev:** "Should I ask users to send logs?"
> **Domain expert:** "Ask for a **Diagnostics Export**. Magnus redacts diagnostic information before sharing."
>
> **Dev:** "Will a **Diagnostics Export** include all of my **Chats**?"
> **Domain expert:** "No. **Chat Transcripts** are excluded by default, and only the **Active Chat's** **Chat Transcript** can be included when the user explicitly chooses it."
>
> **Dev:** "Is this React state or saved desktop data?"
> **Domain expert:** "Saved desktop data belongs to **App Data**. React component state is not domain language."
>
> **Dev:** "Will exporting **App Data** include **Provider API Keys**?"
> **Domain expert:** "No. **Provider API Keys** live in the **System Secret Store**, not in **App Data**."
>
> **Dev:** "Should we call `https://llm-proxy.edgez.live/` a proxy in the UI?"
> **Domain expert:** "No - it is a **Custom Provider** named External Anthropic Proxy. Proxy is an infrastructure detail."
>
> **Dev:** "Is External Anthropic Proxy the same thing as Anthropic?"
> **Domain expert:** "No. It is a **Custom Provider** that uses the Anthropic **Provider Protocol**."
>
> **Dev:** "Does a **Chat** have a model?"
> **Domain expert:** "Not as a persisted Chat-level field. The UI has a **Selected Model**, and each **Model Message** records the **Model** that produced it."
>
> **Dev:** "Does **Provider Settings** include the **Provider API Key**?"
> **Domain expert:** "No. **Provider Settings** are non-secret. The **Provider API Key** is a Provider-owned secret."
>
> **Dev:** "Is the **Default Provider** always built in?"
> **Domain expert:** "No. Any configured **Provider** can be the **Default Provider**."
>
> **Dev:** "Can I switch a **Chat** from Anthropic to OpenAI halfway through?"
> **Domain expert:** "No. A **Chat** has one **Provider**. Multi-provider discussions belong to future **Agent** workflows."
>
> **Dev:** "What happens if a **Chat's** **Provider** is deleted?"
> **Domain expert:** "The **Chat** should remain visible but become a **Blocked Chat** that cannot accept new **User Messages**."
>
> **Dev:** "Is the streamed text already a saved message?"
> **Domain expert:** "The **Model Streaming Response** is the live delivery. The saved artifact in the **Chat** is the **Model Message**."
>
> **Dev:** "Should I call the user's text a prompt?"
> **Domain expert:** "Inside a **Chat**, call it a **User Message**. Prompt is too broad for future system and agent instructions."
>
> **Dev:** "Is a Skill the same thing as a **System Instruction**?"
> **Domain expert:** "No. A Skill may contribute instructions, but the **System Instruction** is the instruction text that shapes model or agent behavior."
>
> **Dev:** "Is an **Agent Loop** just a workflow with repeated steps?"
> **Domain expert:** "No. An **Agent Workflow** follows predefined steps. An **Agent Loop** is a discussion table where **Agents** deliberate until they agree."
>
> **Dev:** "Should multi-agent work be added into the normal **Chat** screen?"
> **Domain expert:** "No. Multi-agent work belongs in **Agent Studio**, which can have its own interaction model."
>
> **Dev:** "Should we call a filesystem helper a capability?"
> **Domain expert:** "If it comes from MCP, call it an **MCP Tool**. Capability is reserved for platform permissions."
>
> **Dev:** "Is `stream_message` the same kind of command as an **MCP Tool**?"
> **Domain expert:** "No. `stream_message` is exposed to the frontend as a **Tauri Command**. An **MCP Tool** is callable by a Model or Agent."
>
> **Dev:** "Does this **MCP Tool** belong only to the active **Chat**?"
> **Domain expert:** "No. Once the **MCP Server** is a **Connected MCP Server**, its **MCP Tools** are globally available to all **Chats** for the current runtime."
>
> **Dev:** "I saved an **MCP Server**. Can models use its tools now?"
> **Domain expert:** "Only after it becomes a **Connected MCP Server**. A **Configured MCP Server** is just the saved launch definition."
>
> **Dev:** "Will my **Connected MCP Servers** reconnect automatically after restarting Magnus?"
> **Domain expert:** "Not currently. **Configured MCP Servers** persist, but **Connected MCP Server** state is runtime-only."
>
> **Dev:** "If one **Chat** uses a GitHub **MCP Tool**, does every other **Chat** learn that result?"
> **Domain expert:** "No. The **MCP Tool** is globally available, but the **Tool Call** result belongs to the **Chat** that invoked it."

## Flagged ambiguities

- "session" was used for both user conversations and runtime diagnostics - resolved: use **Chat** for the conversation artifact and **Diagnostic Session** for runtime diagnostics.
- "assistant message" was too provider-shaped - resolved: use **Model Message** for model-produced Chat content.
- "prompt" was too broad for Chat content - resolved: use **User Message** for user-written Chat content.
- "system prompt" was provider-shaped and overloaded - resolved: use **System Instruction** for future non-user behavior instructions.
- "work session" is reserved for a possible future project/workflow concept and should not be used for current chats.
- "multi-provider chat" was used as a possible future direction - resolved: current **Chats** are single-provider; future multi-provider workflows should be modeled through **Agents**.
- "agent" should not describe current **Chats** - resolved: **Agent** is future-only until **Agent Studio** exists.
- "LLM Loop" was used for multi-model deliberation - resolved: use **Agent Loop** because the participating Agents own their Provider and Model choices.
- "capability" can conflict with Tauri permissions - resolved: use **MCP Tool** for model-callable MCP behavior.
- "command" can mean too many things - resolved: use **Tauri Command** for frontend-to-backend operations and **MCP Tool** for model-callable behavior.
- "MCP Server" can mean saved definition or open connection - resolved: use **Configured MCP Server** for saved launch details and **Connected MCP Server** for an active runtime connection.
- "connected MCP" can sound persisted - resolved: only **Configured MCP Servers** persist; **Connected MCP Server** state is runtime-only for now.
- "global MCP" was ambiguous between tool availability and shared context - resolved: **MCP Tools** are globally available to **Chats**, but **Tool Call** results are not shared between **Chats** by default.
- "Chat Transcript" may later expand to include tool activity - current behavior: **Chat Transcripts** contain only **User Messages** and **Model Messages**.
- "agent flow" was used for the future multi-agent product area - resolved: use **Agent Studio**.
- "local state" can mean React component state or desktop persistence - resolved: use **App Data** for Magnus-owned persisted data.
- "API key" was too generic - resolved: use **Provider API Key** because credentials belong to a specific **Provider**.
- "provider configuration" was inconsistent with **Settings** - resolved: use **Provider Settings**.
- "protocol" was too generic - resolved: use **Provider Protocol** for provider API formats such as Anthropic, OpenAI, and Google.
- "default provider" was sometimes confused with built-in providers - resolved: any configured **Provider** can be the **Default Provider**.
- "deleted provider" behavior is not implemented yet - intended behavior: existing **Chats** remain visible but become **Blocked Chats** when their **Provider** is unavailable.
- "proxy" currently means a user-configured LLM endpoint - resolved: model it as a **Custom Provider**; do not introduce **Network Proxy** until Magnus supports separate HTTP/HTTPS/SOCKS routing configuration.
- "Chat model" suggests a persisted Chat-level model - current temporary behavior: use **Selected Model** for the UI choice and record **Model** on **Model Messages**.
- "settings contains keys" is incorrect - resolved: **Settings** contains non-secret **Provider Settings**; **Provider API Keys** live in the **System Secret Store**.
- "logs" was too narrow - resolved: use **Diagnostics**, **Diagnostic Event**, and **Diagnostics Export**.
