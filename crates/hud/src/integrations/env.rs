use crate::model::EnvSnapshot;

fn var(name: &str) -> Option<String> {
    std::env::var(name).ok()
}

pub(super) fn snapshot() -> EnvSnapshot {
    EnvSnapshot {
        no_color: var("NO_COLOR"),
        force_color: var("FORCE_COLOR"),
        clicolor: var("CLICOLOR"),
        clicolor_force: var("CLICOLOR_FORCE"),
        colorterm: var("COLORTERM"),
        term: var("TERM"),
        columns: var("COLUMNS"),
        lines: var("LINES"),
    }
}
