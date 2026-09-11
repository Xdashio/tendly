use tracing_subscriber::{fmt, EnvFilter};

/// Initialize structured logging for Tendly.
///
/// CRITICAL PRIVACY DIRECTIVE:
/// Never log raw user activity. Specifically, do NOT log:
/// - Window titles
/// - URLs or browser domains
/// - Process paths or file paths from windows
/// - Keystrokes or input events
/// - Personal activity labels
/// - Classification prompts containing user context
///
/// All default logs must be strictly structural and safe to share in bug reports.
pub fn init_logging() {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("tendly=info,tauri=info,warn"));

    let _ = fmt()
        .with_env_filter(filter)
        .with_target(true)
        .with_thread_ids(false)
        .with_thread_names(false)
        .compact()
        .try_init();
}
