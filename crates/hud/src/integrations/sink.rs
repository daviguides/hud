use std::io::{self, Write};

use crate::model::Stream;

/// Writes `text` to a standard stream and flushes it.
pub(super) fn write(stream: Stream, text: &str) -> io::Result<()> {
    match stream {
        Stream::Stdout => {
            let mut out = io::stdout().lock();
            out.write_all(text.as_bytes())?;
            out.flush()
        }
        Stream::Stderr => {
            let mut out = io::stderr().lock();
            out.write_all(text.as_bytes())?;
            out.flush()
        }
    }
}
