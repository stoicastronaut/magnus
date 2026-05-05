mod capture;
mod event;
mod export;
mod redaction;
mod retention;
mod writer;

pub use capture::{ClientEvent, install_panic_hook, record_result};
pub use event::{
    DiagnosticContext, DiagnosticEvent, DiagnosticKind, DiagnosticLevel,
    DiagnosticSource,
};
pub use export::{
    ExportOptions, ExportResult, diagnostics_summary,
    export_diagnostics_bundle, validate_reveal_path,
};
pub use redaction::{redact_context, redact_message};
pub use writer::{
    Diagnostics, append_crash_event, diagnostics_dir, read_recent_diagnostics,
    start_diagnostics,
};
