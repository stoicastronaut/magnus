# Diagnostics Exports Exclude Chat Transcripts by Default

Magnus excludes Chat transcripts from Diagnostics Exports by default and includes only redacted diagnostic information unless the user explicitly chooses to include the active Chat transcript. This protects private conversation content during troubleshooting while still allowing a user to share the specific Chat context needed to debug provider, model, or tool failures. Diagnostics Exports must not include all Chats implicitly.
