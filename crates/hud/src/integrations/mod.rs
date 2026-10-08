//! The only layer that reads the environment or asks the operating system.

mod env;
mod sink;
mod tty;

use crate::model::{EnvSnapshot, Stream, StreamInfo};

/// Source of the facts the resolver needs. The real one reads the process
/// environment and asks the OS; tests substitute a counting fake.
pub(crate) trait Probe {
    /// The environment variables that matter.
    fn env_snapshot(&self) -> EnvSnapshot;
    /// What the OS reports about one stream: at most one syscall, never a child process.
    fn stream_info(&self, stream: Stream) -> StreamInfo;
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
}

/// Writes `text` to `stream`, flushing, and reports the error instead of hiding it.
pub(crate) fn write(stream: Stream, text: &str) -> std::io::Result<()> {
    sink::write(stream, text)
}
