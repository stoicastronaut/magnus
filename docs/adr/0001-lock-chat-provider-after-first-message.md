# Lock Chat Provider After First Message

Magnus treats a Chat as a single-provider conversation: the Provider is selected before the first message and cannot change once messages exist. Allowing provider changes mid-chat would make persisted history, request replay, diagnostics, tool behavior, and provider-specific message semantics harder to reason about. Future multi-provider workflows should be modeled through Agents, where each Agent can have its own Provider and Model, rather than by switching the Provider inside one Chat.
