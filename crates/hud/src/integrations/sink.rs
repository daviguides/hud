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

/// Writes `text` to a standard stream in as few system calls as it can, and flushes it.
///
/// The standard streams of Rust are line buffered: text that does not end with a newline is cut at
/// its last one, and the rest waits for a flush, which makes two system calls for one frame of a
/// live display. The frame is handed to the kernel in one write instead, after whatever the
/// stream still held has been flushed to keep the order.
#[cfg(unix)]
pub(super) fn write_frame(stream: Stream, text: &str) -> io::Result<()> {
    fn write_all(fd: impl std::os::fd::AsFd, mut bytes: &[u8]) -> io::Result<()> {
        while !bytes.is_empty() {
            match rustix::io::write(&fd, bytes) {
                Ok(0) => return Err(io::ErrorKind::WriteZero.into()),
                Ok(written) => bytes = &bytes[written..],
                Err(rustix::io::Errno::INTR) => {}
                Err(error) => return Err(error.into()),
            }
        }
        Ok(())
    }
    match stream {
        Stream::Stdout => {
            let mut out = io::stdout().lock();
            out.flush()?;
            write_all(&out, text.as_bytes())
        }
        Stream::Stderr => {
            let mut out = io::stderr().lock();
            out.flush()?;
            write_all(&out, text.as_bytes())
        }
    }
}

/// Without a raw write the frame goes through the standard stream.
#[cfg(not(unix))]
pub(super) fn write_frame(stream: Stream, text: &str) -> io::Result<()> {
    write(stream, text)
}
