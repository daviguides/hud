use crate::model::{Stream, StreamInfo};

/// One `ioctl(TIOCGWINSZ)` on the stream: it answers both "is this a terminal" and "how big".
#[cfg(unix)]
pub(super) fn stream_info(stream: Stream) -> StreamInfo {
    let size = match stream {
        Stream::Stdout => rustix::termios::tcgetwinsize(std::io::stdout()),
        Stream::Stderr => rustix::termios::tcgetwinsize(std::io::stderr()),
    };
    match size {
        Ok(ws) => StreamInfo {
            is_tty: true,
            size: (ws.ws_col > 0 && ws.ws_row > 0).then_some((ws.ws_col, ws.ws_row)),
        },
        Err(_) => StreamInfo::default(),
    }
}

/// `GetConsoleMode` answers "is this a console" and `GetConsoleScreenBufferInfo` the size of the
/// visible window, both through safe wrappers (`std` and `terminal_size`).
#[cfg(windows)]
pub(super) fn stream_info(stream: Stream) -> StreamInfo {
    use std::io::IsTerminal;
    let (is_tty, size) = match stream {
        Stream::Stdout => (
            std::io::stdout().is_terminal(),
            terminal_size::terminal_size_of(std::io::stdout()),
        ),
        Stream::Stderr => (
            std::io::stderr().is_terminal(),
            terminal_size::terminal_size_of(std::io::stderr()),
        ),
    };
    StreamInfo {
        is_tty,
        size: size.and_then(|(width, height)| {
            (width.0 > 0 && height.0 > 0).then_some((width.0, height.0))
        }),
    }
}

/// Without a platform size call the size is left to `COLUMNS`, `LINES` and the defaults.
#[cfg(not(any(unix, windows)))]
pub(super) fn stream_info(stream: Stream) -> StreamInfo {
    use std::io::IsTerminal;
    let is_tty = match stream {
        Stream::Stdout => std::io::stdout().is_terminal(),
        Stream::Stderr => std::io::stderr().is_terminal(),
    };
    StreamInfo { is_tty, size: None }
}
