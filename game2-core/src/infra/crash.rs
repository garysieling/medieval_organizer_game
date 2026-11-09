//! A panic hook that files a GitHub issue for the crash (#941).
//!
//! `install()` wraps the default hook at the top of every binary's main:
//! the panic still prints like always, then `gh issue create` tries to
//! file it. No gh on PATH, no auth, no network - the call just fails and
//! the game dies the way it always did; the hook never crashes itself.
//!
//! `GAME2_CRASH_DRY=1` prints the would-be issue instead of filing, and
//! `GAME2_CRASH_TEST=1` panics deliberately right after installing - the
//! two together check the path end to end without touching the repo.

use std::panic::PanicHookInfo;
use std::sync::atomic::{AtomicBool, Ordering};

static INSTALLED: AtomicBool = AtomicBool::new(false);

/// The default hook, wrapped by a best-effort `gh issue create`. Safe to
/// call more than once - only the first installs.
pub fn install() {
    if INSTALLED.swap(true, Ordering::SeqCst) { return; }
    let prev = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        prev(info);
        report(info);
    }));
    if std::env::var_os("GAME2_CRASH_TEST").is_some() {
        panic!("deliberate crash test (GAME2_CRASH_TEST)");
    }
}

fn report(info: &PanicHookInfo<'_>) {
    let bin = std::env::current_exe().ok()
        .and_then(|p| p.file_stem().map(|s| s.to_string_lossy().into_owned()))
        .unwrap_or_else(|| "game2".into());
    let where_at = info.location()
        .map(|l| format!("{}:{}:{}", l.file(), l.line(), l.column()))
        .unwrap_or_else(|| "unknown".into());
    let title = truncate(&format!("crash: {bin}: {}", panic_message(info)), 200);
    let bt = truncate(&std::backtrace::Backtrace::force_capture().to_string(), 4000);
    let body = format!(
        "Panic in `{bin}` ({}/{}, v{})\n\nat `{where_at}`\n\n```\n{}\n```\n\nBacktrace:\n```\n{bt}\n```",
        std::env::consts::OS, std::env::consts::ARCH,
        env!("CARGO_PKG_VERSION"), panic_message(info));
    file_issue(&title, &body);
}

/// What the panic said: the &str/String payload, or a placeholder for a
/// payload we can't read
fn panic_message(info: &PanicHookInfo<'_>) -> String {
    if let Some(s) = info.payload().downcast_ref::<&str>() { return s.to_string(); }
    if let Some(s) = info.payload().downcast_ref::<String>() { return s.clone(); }
    "panic (message not readable)".into()
}

fn truncate(s: &str, max: usize) -> String {
    let mut t: String = s.chars().take(max).collect();
    if s.chars().count() > max { t.push_str("..."); }
    t
}

/// `gh issue create` on the repo - fire and forget: a missing gh, no
/// auth, a dead network all look the same to us and don't matter
fn file_issue(title: &str, body: &str) {
    if std::env::var_os("GAME2_CRASH_DRY").is_some() {
        eprintln!("[crash hook] dry run - would file:\n  title: {title}\n{body}");
        return;
    }
    let _ = std::process::Command::new("gh")
        .args(["issue", "create", "-R", "garysieling/game2",
               "--label", "bug", "--title", title, "--body", body])
        .output();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn truncate_holds_the_cap() {
        let short = truncate("abc", 10);
        assert_eq!(short, "abc");
        let long = truncate(&"x".repeat(300), 200);
        assert_eq!(long.chars().count(), 203);
        assert!(long.ends_with("..."));
    }
}
