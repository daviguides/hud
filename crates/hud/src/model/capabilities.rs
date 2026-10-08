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
#[non_exhaustive]
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
#[non_exhaustive]
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

impl EnvSnapshot {
    /// A snapshot with no variable set.
    pub fn new() -> EnvSnapshot {
        EnvSnapshot::default()
    }

    /// Sets `NO_COLOR`.
    #[must_use]
    pub fn no_color(mut self, value: impl Into<String>) -> EnvSnapshot {
        self.no_color = Some(value.into());
        self
    }

    /// Sets `FORCE_COLOR`.
    #[must_use]
    pub fn force_color(mut self, value: impl Into<String>) -> EnvSnapshot {
        self.force_color = Some(value.into());
        self
    }

    /// Sets `CLICOLOR`.
    #[must_use]
    pub fn clicolor(mut self, value: impl Into<String>) -> EnvSnapshot {
        self.clicolor = Some(value.into());
        self
    }

    /// Sets `CLICOLOR_FORCE`.
    #[must_use]
    pub fn clicolor_force(mut self, value: impl Into<String>) -> EnvSnapshot {
        self.clicolor_force = Some(value.into());
        self
    }

    /// Sets `COLORTERM`.
    #[must_use]
    pub fn colorterm(mut self, value: impl Into<String>) -> EnvSnapshot {
        self.colorterm = Some(value.into());
        self
    }

    /// Sets `TERM`.
    #[must_use]
    pub fn term(mut self, value: impl Into<String>) -> EnvSnapshot {
        self.term = Some(value.into());
        self
    }

    /// Sets `COLUMNS`.
    #[must_use]
    pub fn columns(mut self, value: impl Into<String>) -> EnvSnapshot {
        self.columns = Some(value.into());
        self
    }

    /// Sets `LINES`.
    #[must_use]
    pub fn lines(mut self, value: impl Into<String>) -> EnvSnapshot {
        self.lines = Some(value.into());
        self
    }
}

/// What the operating system reports about one output stream.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[non_exhaustive]
pub struct StreamInfo {
    /// The stream is attached to a terminal.
    pub is_tty: bool,
    /// Window size as `(columns, rows)`, when the terminal reports one.
    pub size: Option<(u16, u16)>,
}

impl StreamInfo {
    /// A stream that is not a terminal and reports no size.
    pub fn new() -> StreamInfo {
        StreamInfo::default()
    }

    /// A terminal that reports a window of `columns` by `rows`.
    pub fn terminal(columns: u16, rows: u16) -> StreamInfo {
        StreamInfo {
            is_tty: true,
            size: Some((columns, rows)),
        }
    }

    /// Whether the stream is attached to a terminal.
    #[must_use]
    pub fn is_tty(mut self, is_tty: bool) -> StreamInfo {
        self.is_tty = is_tty;
        self
    }

    /// The window size the terminal reports, `columns` by `rows`.
    #[must_use]
    pub fn size(mut self, columns: u16, rows: u16) -> StreamInfo {
        self.size = Some((columns, rows));
        self
    }
}

/// What the output stream can show. Resolved once per process and stream.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct Capabilities {
    /// Color depth; [`ColorSystem::None`] when color is off.
    pub color_system: ColorSystem,
    /// Bold, italic and underline may be emitted. `NO_COLOR` keeps this on.
    pub attributes: bool,
    /// The stream is attached to a terminal.
    pub is_tty: bool,
    /// A display may redraw in place with cursor control: a terminal, or a stream forced to
    /// behave as one, that is not `TERM=dumb`.
    pub interactive: bool,
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
