/// What a Windows console reports that the portable variables do not: whether it accepts ANSI
/// sequences and which host program it is. Plain data, read once per process.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[cfg_attr(not(windows), allow(dead_code))]
pub(crate) struct WindowsFacts {
    /// Virtual terminal processing is on for the console (it was enabled or already was).
    pub(crate) vt_enabled: bool,
    /// `WT_SESSION` is set: the process runs inside Windows Terminal.
    pub(crate) wt_session: bool,
    /// `ConEmuANSI`: `ON` when ConEmu translates ANSI sequences.
    pub(crate) conemu_ansi: Option<String>,
    /// `ANSICON` is set: the ANSICON injector translates ANSI sequences.
    pub(crate) ansicon: bool,
    /// `TERM_PROGRAM`: the host terminal program, such as `vscode`.
    pub(crate) term_program: Option<String>,
}
