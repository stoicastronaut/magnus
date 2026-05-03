# Store Provider API Keys in the System Secret Store

Magnus stores Provider API Keys in the operating system's System Secret Store, keyed by Provider id, while App Data stores only non-secret Settings and Provider Settings. Keeping credentials out of App Data makes diagnostics, exports, backups, and settings files safer by default, even though it makes Provider setup less portable across machines than a single self-contained settings file.
