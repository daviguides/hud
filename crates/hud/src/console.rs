//! The composition point: connects the pure services to the real environment and the
//! standard streams. Each stream is resolved once per process.

use std::fmt;
use std::io;
use std::sync::OnceLock;

use crate::integrations::{self, Probe, SystemProbe};
use crate::model::{
    Capabilities, ColorSystem, Measure, Panel, Renderable, Segment, Stream, Table, Text, Tree,
};
use crate::services::layout::measure_text;
use crate::services::panel::{measure_panel, render_panel};
use crate::services::render::{crop_lines, render_text, render_text_ending, to_ansi, to_plain};
use crate::services::resolve::resolve;
use crate::services::table::{measure_table, render_table};
use crate::services::tree::{measure_tree, render_tree};

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

/// Where output goes and what it can show: color depth, attributes and width.
///
/// A console made with [`Console::stdout`] or [`Console::stderr`] takes its profile from the
/// environment, resolved once per process. A [`ConsoleBuilder`] overrides any part of it, which
/// is how tests and tools get the same bytes on every machine.
///
/// ```
/// use hud::{ColorSystem, Console};
///
/// let console = Console::builder()
///     .width(40)
///     .color_system(ColorSystem::TrueColor)
///     .attributes(true)
///     .build();
/// assert_eq!(console.render_to_string("[bold]ok[/]"), "\x1b[1mok\x1b[0m\n");
/// ```
#[derive(Clone, Debug)]
pub struct Console {
    caps: Capabilities,
    stream: Stream,
}

impl Console {
    /// A console on standard output with the profile the environment gives it.
    pub fn stdout() -> Console {
        Console {
            caps: capabilities(Stream::Stdout),
            stream: Stream::Stdout,
        }
    }

    /// A console on standard error with the profile the environment gives it.
    pub fn stderr() -> Console {
        Console {
            caps: capabilities(Stream::Stderr),
            stream: Stream::Stderr,
        }
    }

    /// A builder that starts from the standard output profile and overrides parts of it.
    pub fn builder() -> ConsoleBuilder {
        ConsoleBuilder {
            console: Console::stdout(),
        }
    }

    /// What this console can show.
    pub fn capabilities(&self) -> &Capabilities {
        &self.caps
    }

    /// Prints `renderable` and ignores write errors: a closed pipe (`prog | head`) ends the
    /// output quietly. A string is read as markup; markup that does not parse prints as it is.
    /// Use [`Console::try_print`] to see the error.
    pub fn print<R: Renderable + ?Sized>(&self, renderable: &R) {
        let _ = self.try_print(renderable);
    }

    /// Prints `renderable` and returns the write error, if any.
    pub fn try_print<R: Renderable + ?Sized>(&self, renderable: &R) -> io::Result<()> {
        integrations::write(self.stream, &self.render_to_string(renderable))
    }

    /// What [`Console::print`] would write: the bytes for this console's profile.
    pub fn render_to_string<R: Renderable + ?Sized>(&self, renderable: &R) -> String {
        let width = usize::from(self.caps.width);
        to_ansi(&crop_lines(renderable.render(width), width), &self.caps)
    }

    /// The same output with no escape sequences.
    pub fn render_to_plain<R: Renderable + ?Sized>(&self, renderable: &R) -> String {
        let width = usize::from(self.caps.width);
        to_plain(&crop_lines(renderable.render(width), width))
    }
}

/// Overrides parts of a console's profile; made by [`Console::builder`].
#[derive(Clone, Debug)]
pub struct ConsoleBuilder {
    console: Console,
}

impl ConsoleBuilder {
    /// Which standard stream [`Console::print`] writes to (the profile is kept).
    #[must_use]
    pub fn stream(mut self, stream: Stream) -> ConsoleBuilder {
        self.console.stream = stream;
        self
    }

    /// The width in cells lines are wrapped to.
    #[must_use]
    pub fn width(mut self, width: u16) -> ConsoleBuilder {
        self.console.caps.width = width;
        self
    }

    /// The height in rows.
    #[must_use]
    pub fn height(mut self, height: u16) -> ConsoleBuilder {
        self.console.caps.height = height;
        self
    }

    /// The color depth; [`ColorSystem::None`] drops colors only.
    #[must_use]
    pub fn color_system(mut self, color_system: ColorSystem) -> ConsoleBuilder {
        self.console.caps.color_system = color_system;
        self
    }

    /// Whether bold, italic, underline and the other attributes (and links) are emitted.
    #[must_use]
    pub fn attributes(mut self, attributes: bool) -> ConsoleBuilder {
        self.console.caps.attributes = attributes;
        self
    }

    /// No escape sequences at all: no color and no attributes.
    #[must_use]
    pub fn plain(self) -> ConsoleBuilder {
        self.color_system(ColorSystem::None).attributes(false)
    }

    /// Finishes the console.
    pub fn build(self) -> Console {
        self.console
    }
}

impl Renderable for Text {
    fn render(&self, width: usize) -> Vec<Segment> {
        render_text(self, width)
    }

    fn measure(&self, _max_width: usize) -> Measure {
        measure_text(self).measure()
    }
}

impl Renderable for Table {
    fn render(&self, width: usize) -> Vec<Segment> {
        render_table(self, width)
    }

    fn measure(&self, max_width: usize) -> Measure {
        measure_table(self, max_width)
    }
}

impl Renderable for Panel {
    fn render(&self, width: usize) -> Vec<Segment> {
        render_panel(self, width)
    }

    fn measure(&self, max_width: usize) -> Measure {
        measure_panel(self, max_width)
    }
}

impl Renderable for Tree {
    fn render(&self, width: usize) -> Vec<Segment> {
        render_tree(self, width)
    }

    fn measure(&self, max_width: usize) -> Measure {
        measure_tree(self, max_width)
    }
}

impl Renderable for str {
    fn render(&self, width: usize) -> Vec<Segment> {
        markup_or_plain(self).render(width)
    }

    fn measure(&self, max_width: usize) -> Measure {
        markup_or_plain(self).measure(max_width)
    }
}

impl Renderable for String {
    fn render(&self, width: usize) -> Vec<Segment> {
        self.as_str().render(width)
    }

    fn measure(&self, max_width: usize) -> Measure {
        self.as_str().measure(max_width)
    }
}

impl<T: Renderable + ?Sized> Renderable for &T {
    fn render(&self, width: usize) -> Vec<Segment> {
        (**self).render(width)
    }

    fn measure(&self, max_width: usize) -> Measure {
        (**self).measure(max_width)
    }
}

/// Markup read as text; markup that does not parse is printed as it is.
fn markup_or_plain(markup: &str) -> Text {
    Text::from_markup(markup).unwrap_or_else(|_| Text::new(markup))
}

/// Renders for the standard output profile, without the trailing newline `print` adds, so
/// `println!("{text}")` and `format!("{text}")` give the styled text.
impl fmt::Display for Text {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let caps = capabilities(Stream::Stdout);
        let segments = render_text_ending(self, usize::from(caps.width), "");
        f.write_str(&to_ansi(&segments, &caps))
    }
}

/// Renders for the standard output profile without the final newline, so `println!("{table}")`
/// prints the table once.
impl fmt::Display for Table {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let caps = capabilities(Stream::Stdout);
        let width = usize::from(caps.width);
        let ansi = to_ansi(&crop_lines(render_table(self, width), width), &caps);
        f.write_str(ansi.strip_suffix('\n').unwrap_or(&ansi))
    }
}

/// Renders for the standard output profile without the final newline, like [`Table`].
impl fmt::Display for Panel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        display_block(f, &render_panel(self, display_width()))
    }
}

/// Renders for the standard output profile without the final newline, like [`Table`].
impl fmt::Display for Tree {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        display_block(f, &render_tree(self, display_width()))
    }
}

fn display_width() -> usize {
    usize::from(capabilities(Stream::Stdout).width)
}

fn display_block(f: &mut fmt::Formatter<'_>, segments: &[Segment]) -> fmt::Result {
    let caps = capabilities(Stream::Stdout);
    let width = usize::from(caps.width);
    let ansi = to_ansi(&crop_lines(segments.to_vec(), width), &caps);
    f.write_str(ansi.strip_suffix('\n').unwrap_or(&ansi))
}

/// Used by the `println!` macros: prints markup, newline included.
#[doc(hidden)]
pub fn print_markup(stream: Stream, markup: &str) {
    let console = match stream {
        Stream::Stdout => Console::stdout(),
        Stream::Stderr => Console::stderr(),
    };
    console.print(markup);
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
