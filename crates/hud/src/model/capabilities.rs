/// Which standard stream output goes to.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Stream {
    /// Standard output.
    Stdout,
    /// Standard error.
    Stderr,
}

/// How many colors the terminal can show.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ColorSystem {
    /// No color at all.
    None,
    /// The 16 standard ANSI colors.
    Standard,
    /// The 256-color palette.
    EightBit,
    /// 24-bit color.
    TrueColor,
}

/// The environment variables that decide capabilities, captured as text.
///
/// An absent variable is `None`. An empty value is kept as `Some("")` because
/// `NO_COLOR` and `FORCE_COLOR` treat empty as unset.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct EnvSnapshot {
    /// `NO_COLOR`: any non-empty value removes color (no-color.org).
    pub no_color: Option<String>,
    /// `FORCE_COLOR`: a non-empty value other than `0` keeps styling on.
    pub force_color: Option<String>,
    /// `CLICOLOR`: `0` turns styling off on a terminal (bixense.com/clicolors).
    pub clicolor: Option<String>,
    /// `CLICOLOR_FORCE`: a non-empty value other than `0` keeps styling on.
    pub clicolor_force: Option<String>,
    /// `COLORTERM`: `truecolor` or `24bit` announce 24-bit color.
    pub colorterm: Option<String>,
    /// `TERM`: the terminal type.
    pub term: Option<String>,
    /// `COLUMNS`: overrides the terminal width.
    pub columns: Option<String>,
    /// `LINES`: overrides the terminal height.
    pub lines: Option<String>,
}

/// What the operating system reports about one output stream.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct StreamInfo {
    /// The stream is attached to a terminal.
    pub is_tty: bool,
    /// Window size as `(columns, rows)`, when the terminal reports one.
    pub size: Option<(u16, u16)>,
}

/// What the output stream can show. Resolved once per process and stream.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Capabilities {
    /// Color depth; [`ColorSystem::None`] when color is off.
    pub color_system: ColorSystem,
    /// Bold, italic and underline may be emitted. `NO_COLOR` keeps this on.
    pub attributes: bool,
    /// The stream is attached to a terminal.
    pub is_tty: bool,
    /// Width in cells.
    pub width: u16,
    /// Height in rows.
    pub height: u16,
}

impl Capabilities {
    /// Any escape sequence at all may be emitted.
    pub fn emits_escapes(&self) -> bool {
        self.attributes || self.color_system != ColorSystem::None
    }
}
