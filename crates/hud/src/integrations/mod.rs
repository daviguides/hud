//! The only layer that reads the environment or asks the operating system.

mod env;
mod sink;
mod tty;
#[cfg(windows)]
mod winfacts;

use crate::model::{EnvSnapshot, Stream, StreamInfo, WindowsFacts};

/// Source of the facts the resolver needs. The real one reads the process
/// environment and asks the OS; tests substitute a counting fake.
pub(crate) trait Probe {
    /// The environment variables that matter.
    fn env_snapshot(&self) -> EnvSnapshot;
    /// What the OS reports about one stream: a small constant number of OS calls, never a child
    /// process.
    fn stream_info(&self, stream: Stream) -> StreamInfo;
    /// What a Windows console reports; `None` on every other platform.
    fn windows_facts(&self) -> Option<WindowsFacts> {
        None
    }
}

/// Probe backed by the real process environment and standard streams.
pub(crate) struct SystemProbe;

impl Probe for SystemProbe {
    fn env_snapshot(&self) -> EnvSnapshot {
        env::snapshot()
    }

    fn stream_info(&self, stream: Stream) -> StreamInfo {
        tty::stream_info(stream)
    }

    #[cfg(windows)]
    fn windows_facts(&self) -> Option<WindowsFacts> {
        Some(winfacts::facts())
    }
}

/// Writes `text` to `stream`, flushing, and reports the error instead of hiding it.
pub(crate) fn write(stream: Stream, text: &str) -> std::io::Result<()> {
    sink::write(stream, text)
}

/// Writes a frame of a live display to `stream` in one system call where the platform allows it.
pub(crate) fn write_frame(stream: Stream, text: &str) -> std::io::Result<()> {
    sink::write_frame(stream, text)
}
