use std::sync::OnceLock;

use crate::model::WindowsFacts;

fn var(name: &str) -> Option<String> {
    std::env::var(name).ok()
}

/// Virtual terminal processing is switched on once per process. The console mode stays on for
/// the life of the process, as every ANSI-capable Windows program leaves it.
fn vt_enabled() -> bool {
    static VT: OnceLock<bool> = OnceLock::new();
    *VT.get_or_init(|| enable_ansi_support::enable_ansi_support().is_ok())
}

pub(super) fn facts() -> WindowsFacts {
    WindowsFacts {
        vt_enabled: vt_enabled(),
        wt_session: var("WT_SESSION").is_some_and(|v| !v.is_empty()),
        conemu_ansi: var("ConEmuANSI"),
        ansicon: var("ANSICON").is_some_and(|v| !v.is_empty()),
        term_program: var("TERM_PROGRAM"),
    }
}
