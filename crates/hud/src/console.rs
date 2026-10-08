//! The composition point: connects the pure services to the real environment and the
//! standard streams. Each stream is resolved once per process.

use std::fmt;
use std::io;
use std::process::ExitCode;
use std::sync::OnceLock;

use crate::integrations::{self, Probe, SystemProbe};
use crate::model::{
    Capabilities, ColorSystem, Columns, ErrorReport, Format, Layout, Measure, Node, Padding, Panel,
    Renderable, Segment, Stream, Table, Text, Tree,
};
use crate::services::columns::render_columns;
use crate::services::layout::measure_text;
use crate::services::node::{
    columns_node, error_node, layout_node, markup_node, padding_node, panel_node, table_node,
    text_node, tree_node,
};
use crate::services::panel::{
    measure_padding, measure_panel, render_padding, render_panel, render_panel_in,
};
use crate::services::render::{
    crop_lines, render_text, render_text_ending, to_ansi, to_plain, without_control_introducers,
};
use crate::services::report::{measure_report, render_report};
use crate::services::resolve::resolve;
use crate::services::split::render_layout;
use crate::services::table::{measure_table, render_table};
use crate::services::tree::{measure_tree, render_tree};
use crate::services::winenv::apply_windows;

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
        *cell.get_or_init(|| {
            let info = self.probe.stream_info(stream);
            let mut env = self.probe.env_snapshot();
            if let Some(facts) = self.probe.windows_facts() {
                env = apply_windows(env, &facts, info.is_tty);
            }
            resolve(&env, info)
        })
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
    format: Format,
}

impl Console {
    /// A console on standard output with the profile the environment gives it.
    pub fn stdout() -> Console {
        Console {
            caps: capabilities(Stream::Stdout),
            stream: Stream::Stdout,
            format: integrations::env_format(),
        }
    }

    /// A console on standard error with the profile the environment gives it.
    pub fn stderr() -> Console {
        Console {
            caps: capabilities(Stream::Stderr),
            stream: Stream::Stderr,
            format: integrations::env_format(),
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

    /// The width in cells lines are wrapped to: Rich's `Console.width`.
    pub fn width(&self) -> usize {
        usize::from(self.caps.width)
    }

    /// Whether the console writes to a terminal, or is forced to act as one: Rich's
    /// `Console.is_terminal`.
    pub fn is_terminal(&self) -> bool {
        self.caps.is_tty || self.caps.interactive
    }

    /// The color depth: Rich's `Console.color_system`. [`ColorSystem::None`] when color is off.
    pub fn color_system(&self) -> ColorSystem {
        self.caps.color_system
    }

    /// Whether color is off: Rich's `Console.no_color`. Bold, italic and underline can still
    /// be on; `NO_COLOR` removes color only.
    pub fn no_color(&self) -> bool {
        self.caps.color_system == ColorSystem::None
    }

    /// Whether the console acts as a terminal although its output is not one (`FORCE_COLOR`,
    /// `CLICOLOR_FORCE` or [`ConsoleBuilder::force_terminal`]): Rich's `force_terminal`.
    pub fn force_terminal(&self) -> bool {
        self.caps.interactive && !self.caps.is_tty
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

    /// What [`Console::print`] would write: the bytes for this console's profile in this
    /// console's [`Format`].
    pub fn render_to_string<R: Renderable + ?Sized>(&self, renderable: &R) -> String {
        self.render_as(renderable, self.format)
    }

    /// What `renderable` is in `format`, whatever this console's own format is: styled text for
    /// this console's profile, text with no escape sequence, or the JSON document. Plain text
    /// also leaves out the escape character and the C1 controls found in the data itself, so it
    /// is safe to write to a log or a terminal; [`Console::render_to_plain`] keeps the data as
    /// it is and removes only what hud adds.
    ///
    /// ```
    /// use hud::{Console, Format};
    ///
    /// let console = Console::builder().width(12).build();
    /// assert_eq!(console.render_as("[bold]ok[/]", Format::Plain), "ok\n");
    /// assert!(console.render_as("ok", Format::Json).contains("\"text\": \"ok\""));
    /// ```
    pub fn render_as<R: Renderable + ?Sized>(&self, renderable: &R, format: Format) -> String {
        match format {
            Format::Rich => {
                let width = usize::from(self.caps.width);
                to_ansi(
                    &crop_lines(renderable.render_with(width, &self.caps), width),
                    &self.caps,
                )
            }
            Format::Plain => without_control_introducers(&self.render_to_plain(renderable)),
            Format::Json => renderable.node().to_json(),
        }
    }

    /// The format this console writes in: [`Format::Rich`] unless the environment variable
    /// `HUD_FORMAT` or [`ConsoleBuilder::format`] chose another.
    pub fn format(&self) -> Format {
        self.format
    }

    /// The same output with no escape sequences.
    pub fn render_to_plain<R: Renderable + ?Sized>(&self, renderable: &R) -> String {
        let width = usize::from(self.caps.width);
        to_plain(&crop_lines(
            renderable.render_with(width, &self.caps),
            width,
        ))
    }

    /// The stream this console prints to.
    pub(crate) fn stream(&self) -> Stream {
        self.stream
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

    /// The format the console writes in, over whatever `HUD_FORMAT` says.
    ///
    /// ```
    /// use hud::{Console, Format};
    ///
    /// let console = Console::builder().format(Format::Json).build();
    /// assert_eq!(console.format(), Format::Json);
    /// ```
    #[must_use]
    pub fn format(mut self, format: Format) -> ConsoleBuilder {
        self.console.format = format;
        self
    }

    /// Makes the console act as a terminal (redrawing in place) or not, whatever the stream is.
    #[must_use]
    pub fn force_terminal(mut self, force: bool) -> ConsoleBuilder {
        self.console.caps.interactive = force;
        if !force {
            self.console.caps.is_tty = false;
        }
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

    fn node(&self) -> Node {
        text_node(self)
    }

    fn measure(&self, _max_width: usize) -> Measure {
        measure_text(self).measure()
    }
}

impl Renderable for Table {
    fn node(&self) -> Node {
        table_node(self)
    }

    fn render(&self, width: usize) -> Vec<Segment> {
        render_table(self, width)
    }

    fn measure(&self, max_width: usize) -> Measure {
        measure_table(self, max_width)
    }
}

impl Renderable for Padding {
    fn node(&self) -> Node {
        padding_node(self)
    }

    fn render(&self, width: usize) -> Vec<Segment> {
        render_padding(self, width)
    }

    fn measure(&self, max_width: usize) -> Measure {
        measure_padding(self, max_width)
    }
}

impl Renderable for Panel {
    fn node(&self) -> Node {
        panel_node(self)
    }

    fn render(&self, width: usize) -> Vec<Segment> {
        render_panel(self, width)
    }

    fn render_region(&self, width: usize, height: usize, caps: &Capabilities) -> Vec<Segment> {
        render_panel_in(self, width, Some((height, caps)))
    }

    fn measure(&self, max_width: usize) -> Measure {
        measure_panel(self, max_width)
    }
}

impl Renderable for Tree {
    fn node(&self) -> Node {
        tree_node(self)
    }

    fn render(&self, width: usize) -> Vec<Segment> {
        render_tree(self, width)
    }

    fn measure(&self, max_width: usize) -> Measure {
        measure_tree(self, max_width)
    }
}

impl Renderable for str {
    fn node(&self) -> Node {
        markup_node(self)
    }

    fn render(&self, width: usize) -> Vec<Segment> {
        markup_or_plain(self).render(width)
    }

    fn measure(&self, max_width: usize) -> Measure {
        markup_or_plain(self).measure(max_width)
    }
}

impl Renderable for String {
    fn node(&self) -> Node {
        markup_node(self)
    }

    fn render(&self, width: usize) -> Vec<Segment> {
        self.as_str().render(width)
    }

    fn measure(&self, max_width: usize) -> Measure {
        self.as_str().measure(max_width)
    }
}

impl Renderable for ErrorReport {
    fn node(&self) -> Node {
        error_node(self)
    }

    fn render(&self, width: usize) -> Vec<Segment> {
        render_report(self, width)
    }

    fn measure(&self, max_width: usize) -> Measure {
        measure_report(self, max_width)
    }
}

impl Renderable for Columns {
    fn node(&self) -> Node {
        columns_node(self)
    }

    fn render(&self, width: usize) -> Vec<Segment> {
        render_columns(self, width, None)
    }

    fn render_with(&self, width: usize, caps: &Capabilities) -> Vec<Segment> {
        render_columns(self, width, Some(caps))
    }
}

impl Renderable for Layout {
    fn node(&self) -> Node {
        layout_node(self)
    }

    fn render(&self, width: usize) -> Vec<Segment> {
        let caps = capabilities(Stream::Stdout);
        render_layout(self, width, usize::from(caps.height), &caps)
    }

    fn render_with(&self, width: usize, caps: &Capabilities) -> Vec<Segment> {
        render_layout(self, width, usize::from(caps.height), caps)
    }

    fn render_region(&self, width: usize, height: usize, caps: &Capabilities) -> Vec<Segment> {
        render_layout(self, width, height, caps)
    }
}

impl<T: Renderable + ?Sized> Renderable for &T {
    fn node(&self) -> Node {
        (**self).node()
    }

    fn render(&self, width: usize) -> Vec<Segment> {
        (**self).render(width)
    }

    fn render_with(&self, width: usize, caps: &Capabilities) -> Vec<Segment> {
        (**self).render_with(width, caps)
    }

    fn render_region(&self, width: usize, height: usize, caps: &Capabilities) -> Vec<Segment> {
        (**self).render_region(width, height, caps)
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
impl fmt::Display for Padding {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        display_block(f, &render_padding(self, display_width()))
    }
}

/// Renders for the standard output profile without the final newline, like [`Table`].
impl fmt::Display for Panel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        display_block(f, &render_panel(self, display_width()))
    }
}

/// Renders for the standard output profile without the final newline, like [`Table`].
impl fmt::Display for ErrorReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        display_block(f, &render_report(self, display_width()))
    }
}

/// Renders for the standard output profile without the final newline, like [`Table`].
impl fmt::Display for Tree {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        display_block(f, &render_tree(self, display_width()))
    }
}

/// Renders for the standard output profile without the final newline, like [`Table`].
impl fmt::Display for Columns {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let caps = capabilities(Stream::Stdout);
        display_block(
            f,
            &render_columns(self, usize::from(caps.width), Some(&caps)),
        )
    }
}

/// Renders for the standard output profile without the final newline, like [`Table`]: as many
/// lines as the terminal has rows.
impl fmt::Display for Layout {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let caps = capabilities(Stream::Stdout);
        let segments = render_layout(
            self,
            usize::from(caps.width),
            usize::from(caps.height),
            &caps,
        );
        display_block(f, &segments)
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

/// Ends a program: on `Err` prints the error to standard error as an [`ErrorReport`] and returns
/// a failing exit code, on `Ok` returns success. Any error type works, and a `Box<dyn Error>`
/// or a string too.
///
/// ```no_run
/// use std::process::ExitCode;
///
/// fn run() -> Result<(), std::io::Error> {
///     std::fs::read_to_string("/etc/hud/config.toml").map(|_| ())
/// }
///
/// fn main() -> ExitCode {
///     hud::report(run())
/// }
/// ```
pub fn report<T, E: Into<Box<dyn std::error::Error>>>(result: Result<T, E>) -> ExitCode {
    match result {
        Ok(_) => ExitCode::SUCCESS,
        Err(error) => {
            let error: Box<dyn std::error::Error> = error.into();
            Console::stderr().print(&ErrorReport::from_error(&*error));
            ExitCode::FAILURE
        }
    }
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
