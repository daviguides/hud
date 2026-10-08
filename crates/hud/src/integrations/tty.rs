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

/// Without a platform size call the size is left to `COLUMNS`, `LINES` and the defaults.
#[cfg(not(unix))]
pub(super) fn stream_info(stream: Stream) -> StreamInfo {
    use std::io::IsTerminal;
    let is_tty = match stream {
        Stream::Stdout => std::io::stdout().is_terminal(),
        Stream::Stderr => std::io::stderr().is_terminal(),
    };
    StreamInfo { is_tty, size: None }
}
