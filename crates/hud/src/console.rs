//! The composition point: connects the pure resolver to the real environment and
//! resolves each stream once per process.

use std::sync::OnceLock;

use crate::integrations::{Probe, SystemProbe};
use crate::model::{Capabilities, Stream};
use crate::services::resolve::resolve;

/// Resolves capabilities through a [`Probe`] and remembers the answer per stream.
pub(crate) struct Resolver<P> {
    probe: P,
    stdout: OnceLock<Capabilities>,
    stderr: OnceLock<Capabilities>,
}

impl<P: Probe> Resolver<P> {
    pub(crate) const fn new(probe: P) -> Self {
        Resolver {
            probe,
            stdout: OnceLock::new(),
            stderr: OnceLock::new(),
        }
    }

    pub(crate) fn get(&self, stream: Stream) -> Capabilities {
        let cell = match stream {
            Stream::Stdout => &self.stdout,
            Stream::Stderr => &self.stderr,
        };
        *cell.get_or_init(|| resolve(&self.probe.env_snapshot(), self.probe.stream_info(stream)))
    }
}

static SYSTEM: Resolver<SystemProbe> = Resolver::new(SystemProbe);

/// What `stream` can show, resolved from the environment on first use and cached for the
/// life of the process: one environment read and at most one syscall, no child process.
///
/// ```
/// use hud::{Stream, capabilities};
///
/// let caps = capabilities(Stream::Stdout);
/// assert!(caps.width > 0 && caps.height > 0);
/// ```
pub fn capabilities(stream: Stream) -> Capabilities {
    SYSTEM.get(stream)
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;

    use super::*;
    use crate::model::{EnvSnapshot, StreamInfo};

    #[derive(Default)]
    struct CountingProbe {
        env_reads: Cell<u32>,
        size_queries: Cell<u32>,
    }

    impl Probe for &CountingProbe {
        fn env_snapshot(&self) -> EnvSnapshot {
            self.env_reads.set(self.env_reads.get() + 1);
            EnvSnapshot {
                term: Some("xterm".into()),
                ..EnvSnapshot::default()
            }
        }

        fn stream_info(&self, _stream: Stream) -> StreamInfo {
            self.size_queries.set(self.size_queries.get() + 1);
            StreamInfo {
                is_tty: true,
                size: Some((90, 30)),
            }
        }
    }

    #[test]
    fn size_is_queried_once_per_stream_however_often_it_is_read() {
        let probe = CountingProbe::default();
        let resolver = Resolver::new(&probe);
        for _ in 0..1000 {
            assert_eq!(resolver.get(Stream::Stdout).width, 90);
        }
        assert_eq!(probe.size_queries.get(), 1);
        assert_eq!(probe.env_reads.get(), 1);
        resolver.get(Stream::Stderr);
        resolver.get(Stream::Stderr);
        assert_eq!(probe.size_queries.get(), 2);
    }
}
