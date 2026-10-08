//! Live display: any renderable redrawn in place, and the handles that update it.
//!
//! This is composition, like [`Console`] and [`Progress`](crate::Progress): it connects the pure
//! redraw protocol to the stream.

use std::fmt;
use std::io::Write;
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use std::time::Duration;

use crate::console::Console;
use crate::integrations;
use crate::model::{Body, VerticalOverflow};
use crate::refresh::Refresher;
use crate::services::live::{Screen, fit_height, frame_lines, push_frame};

fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(PoisonError::into_inner)
}

struct Out {
    writer: Option<Box<dyn Write + Send>>,
    screen: Screen,
    started: bool,
    stopped: bool,
    buffer: String,
}

struct Core {
    console: Console,
    interactive: bool,
    transient: bool,
    disable: bool,
    overflow: VerticalOverflow,
    initial: bool,
    renderable: Mutex<Body>,
    out: Mutex<Out>,
}

impl Core {
    /// The current renderable as lines for this console, cut by the overflow rule `overflow`.
    fn frame(&self, overflow: VerticalOverflow) -> Vec<Vec<crate::model::Segment>> {
        let caps = self.console.capabilities();
        let width = usize::from(caps.width);
        let body = lock(&self.renderable).clone();
        let mut lines = frame_lines(body.0.render_with(width, caps), width);
        fit_height(&mut lines, usize::from(caps.height), width, overflow);
        lines
    }

    fn write(&self, out: &mut Out, text: &str) {
        match out.writer.as_mut() {
            Some(writer) => {
                let _ = writer.write_all(text.as_bytes());
                let _ = writer.flush();
            }
            None => {
                let _ = integrations::write(self.console.stream(), text);
            }
        }
    }

    /// Draws the current renderable over the previous frame.
    fn draw(&self, out: &mut Out) {
        let lines = self.frame(self.overflow);
        let height = lines.len();
        let mut buffer = std::mem::take(&mut out.buffer);
        buffer.clear();
        out.screen.rewind(&mut buffer);
        push_frame(&mut buffer, lines, self.console.capabilities());
        out.screen.drawn(height);
        self.write(out, &buffer);
        out.buffer = buffer;
    }

    fn refresh(&self) {
        let mut out = lock(&self.out);
        if self.disable || !self.interactive || !out.started || out.stopped {
            return;
        }
        self.draw(&mut out);
    }

    fn start(&self) {
        let mut out = lock(&self.out);
        if out.started || out.stopped {
            return;
        }
        out.started = true;
        if self.disable || !self.interactive {
            return;
        }
        let mut buffer = String::new();
        out.screen.open(&mut buffer);
        self.write(&mut out, &buffer);
        if self.initial {
            self.draw(&mut out);
        }
    }

    /// Ends the display: on a terminal the last frame is drawn in full, a newline follows and the
    /// cursor comes back (the frame goes away when transient); anywhere else the last frame is
    /// written once, with no newline after it, unless transient.
    fn stop(&self) {
        let mut out = lock(&self.out);
        if !out.started || out.stopped {
            return;
        }
        out.stopped = true;
        if self.disable {
            return;
        }
        let caps = *self.console.capabilities();
        let mut buffer = String::new();
        if self.interactive {
            let lines = self.frame(VerticalOverflow::Visible);
            let height = lines.len();
            out.screen.rewind(&mut buffer);
            push_frame(&mut buffer, lines, &caps);
            out.screen.drawn(height);
            out.screen.close(&mut buffer, self.transient);
        } else if !self.transient {
            push_frame(&mut buffer, self.frame(VerticalOverflow::Visible), &caps);
        }
        self.write(&mut out, &buffer);
    }
}

struct Inner {
    core: Arc<Core>,
    refresher: Mutex<Option<Refresher>>,
    auto_refresh: Option<Duration>,
}

impl Inner {
    fn stop(&self) {
        if let Some(refresher) = lock(&self.refresher).take() {
            refresher.stop();
        }
        self.core.stop();
    }
}

impl Drop for Inner {
    fn drop(&mut self) {
        self.stop();
    }
}

/// A display that redraws in place: show any renderable, change it with [`Live::update`], and
/// the terminal shows the new one where the old one was.
///
/// On a terminal the cursor is hidden while the display runs, each frame is drawn over the one
/// before, and the last stays on the screen (or goes away, for a [`transient`](LiveBuilder::transient)
/// display). A stream that is not a terminal is written once, when the display ends. A `Live`
/// is a cheap handle: clone it, move it to threads, call every method through a shared
/// reference. The display ends when [`Live::stop`] is called or the last handle is dropped.
///
/// A frame taller than the terminal follows the [`VerticalOverflow`] rule.
///
/// ```no_run
/// use hud::{Live, Table};
///
/// let live = Live::new("starting");
/// live.start();
/// for step in 1..=3 {
///     live.update(Table::new().column("step").row([step.to_string()]));
///     live.refresh();
/// }
/// live.stop();
/// ```
#[derive(Clone)]
pub struct Live {
    inner: Arc<Inner>,
}

impl Live {
    /// A display that starts with `renderable`, on standard output, not started yet.
    pub fn new(renderable: impl Into<Body>) -> Live {
        Live::builder().renderable(renderable).build()
    }

    /// A builder to choose the console, the transient behavior and the refresh.
    pub fn builder() -> LiveBuilder {
        LiveBuilder::new()
    }

    /// Starts the display: the cursor is hidden and, if it has a renderable, the first frame is
    /// drawn. Calling it again does nothing.
    pub fn start(&self) {
        self.inner.core.start();
        if let Some(interval) = self.inner.auto_refresh {
            let mut refresher = lock(&self.inner.refresher);
            if refresher.is_none() && lock(&self.inner.core.out).started {
                let core = Arc::clone(&self.inner.core);
                *refresher = Some(Refresher::start("hud-live", interval, move || {
                    core.refresh();
                }));
            }
        }
    }

    /// Replaces what the display shows. Nothing is drawn until the next refresh (the thread
    /// that refreshes by itself, or [`Live::refresh`]).
    pub fn update(&self, renderable: impl Into<Body>) {
        *lock(&self.inner.core.renderable) = renderable.into();
    }

    /// Draws the display now.
    pub fn refresh(&self) {
        self.inner.core.refresh();
    }

    /// Ends the display: the last frame stays on the screen (or goes away, for a transient
    /// display), the cursor comes back, and nothing is drawn afterwards. Calling it again does
    /// nothing.
    pub fn stop(&self) {
        self.inner.stop();
    }

    /// Whether the display has been started and not stopped.
    pub fn is_started(&self) -> bool {
        let out = lock(&self.inner.core.out);
        out.started && !out.stopped
    }
}

impl fmt::Debug for Live {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Live").finish_non_exhaustive()
    }
}

/// Builds a [`Live`]; made by [`Live::builder`].
///
/// ```
/// use hud::{Live, VerticalOverflow};
///
/// let live = Live::builder()
///     .renderable("[bold]working[/]")
///     .transient(true)
///     .vertical_overflow(VerticalOverflow::Crop)
///     .disable(true)
///     .build();
/// live.start();
/// live.stop();
/// ```
pub struct LiveBuilder {
    renderable: Option<Body>,
    console: Console,
    transient: bool,
    disable: bool,
    auto_refresh: bool,
    refresh_per_second: f64,
    overflow: VerticalOverflow,
    writer: Option<Box<dyn Write + Send>>,
    interactive: Option<bool>,
}

impl LiveBuilder {
    fn new() -> LiveBuilder {
        LiveBuilder {
            renderable: None,
            console: Console::stdout(),
            transient: false,
            disable: false,
            auto_refresh: true,
            refresh_per_second: 4.0,
            overflow: VerticalOverflow::Ellipsis,
            writer: None,
            interactive: None,
        }
    }

    /// What the display shows when it starts (default: nothing).
    #[must_use]
    pub fn renderable(mut self, renderable: impl Into<Body>) -> LiveBuilder {
        self.renderable = Some(renderable.into());
        self
    }

    /// The console the display draws on (default: standard output).
    #[must_use]
    pub fn console(mut self, console: Console) -> LiveBuilder {
        self.console = console;
        self
    }

    /// Removes the display from the screen when it ends instead of leaving the last frame.
    #[must_use]
    pub fn transient(mut self, transient: bool) -> LiveBuilder {
        self.transient = transient;
        self
    }

    /// Draws nothing at all.
    #[must_use]
    pub fn disable(mut self, disable: bool) -> LiveBuilder {
        self.disable = disable;
        self
    }

    /// Whether a background thread redraws the display by itself on a terminal (default: yes).
    /// Without it the display draws when it starts, on [`Live::refresh`] and when it ends.
    #[must_use]
    pub fn auto_refresh(mut self, auto_refresh: bool) -> LiveBuilder {
        self.auto_refresh = auto_refresh;
        self
    }

    /// How often the background thread redraws (default 4).
    #[must_use]
    pub fn refresh_per_second(mut self, per_second: f64) -> LiveBuilder {
        self.refresh_per_second = per_second;
        self
    }

    /// What happens to a frame taller than the terminal (default:
    /// [`VerticalOverflow::Ellipsis`]).
    #[must_use]
    pub fn vertical_overflow(mut self, overflow: VerticalOverflow) -> LiveBuilder {
        self.overflow = overflow;
        self
    }

    /// Writes to `writer` instead of the console's stream.
    #[must_use]
    pub fn writer(mut self, writer: impl Write + Send + 'static) -> LiveBuilder {
        self.writer = Some(Box::new(writer));
        self
    }

    /// Forces redrawing in place (`true`) or printing once at the end (`false`), in place of
    /// what the console reports.
    #[must_use]
    pub fn interactive(mut self, interactive: bool) -> LiveBuilder {
        self.interactive = Some(interactive);
        self
    }

    /// Finishes the display.
    pub fn build(self) -> Live {
        let interactive = self
            .interactive
            .unwrap_or(self.console.capabilities().interactive);
        let auto_refresh =
            (self.auto_refresh && interactive && !self.disable && self.refresh_per_second > 0.0)
                .then(|| Duration::from_secs_f64(1.0 / self.refresh_per_second));
        let initial = self.renderable.is_some();
        let core = Arc::new(Core {
            console: self.console,
            interactive,
            transient: self.transient,
            disable: self.disable,
            overflow: self.overflow,
            initial,
            renderable: Mutex::new(self.renderable.unwrap_or_else(|| Body::from(""))),
            out: Mutex::new(Out {
                writer: self.writer,
                screen: Screen::default(),
                started: false,
                stopped: false,
                buffer: String::new(),
            }),
        });
        Live {
            inner: Arc::new(Inner {
                core,
                refresher: Mutex::new(None),
                auto_refresh,
            }),
        }
    }
}

impl fmt::Debug for LiveBuilder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("LiveBuilder").finish_non_exhaustive()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::ColorSystem;

    #[derive(Clone, Default)]
    struct Capture(Arc<Mutex<Vec<u8>>>);

    impl Write for Capture {
        fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
            lock(&self.0).extend_from_slice(buf);
            Ok(buf.len())
        }

        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }

    impl Capture {
        fn text(&self) -> String {
            String::from_utf8_lossy(&lock(&self.0)).into_owned()
        }
    }

    fn console(height: u16) -> Console {
        Console::builder()
            .width(20)
            .height(height)
            .color_system(ColorSystem::None)
            .attributes(false)
            .build()
    }

    fn live(interactive: bool, transient: bool, height: u16) -> (Live, Capture) {
        let capture = Capture::default();
        let live = Live::builder()
            .renderable("one")
            .console(console(height))
            .interactive(interactive)
            .transient(transient)
            .auto_refresh(false)
            .writer(capture.clone())
            .build();
        (live, capture)
    }

    #[test]
    fn a_terminal_redraws_in_place_and_gives_the_cursor_back() {
        let (live, capture) = live(true, false, 10);
        live.start();
        live.update("two\nlines");
        live.refresh();
        live.stop();
        assert_eq!(
            capture.text(),
            "\x1b[?25lone\r\x1b[2Ktwo\nlines\r\x1b[2K\x1b[1A\x1b[2Ktwo\nlines\n\x1b[?25h"
        );
    }

    #[test]
    fn a_transient_display_erases_itself_when_it_ends() {
        let (live, capture) = live(true, true, 10);
        live.start();
        live.stop();
        assert_eq!(
            capture.text(),
            "\x1b[?25lone\r\x1b[2Kone\n\x1b[?25h\r\x1b[1A\x1b[2K"
        );
    }

    #[test]
    fn a_stream_that_is_not_a_terminal_gets_the_last_frame_once_with_no_newline() {
        let (live, capture) = live(false, false, 10);
        live.start();
        live.update("last");
        live.refresh();
        assert_eq!(capture.text(), "");
        live.stop();
        assert_eq!(capture.text(), "last");
        let (live, capture) = (live, capture);
        live.stop();
        assert_eq!(capture.text(), "last");
    }

    #[test]
    fn a_transient_display_on_a_stream_that_is_not_a_terminal_writes_nothing() {
        let (live, capture) = live(false, true, 10);
        live.start();
        live.stop();
        assert_eq!(capture.text(), "");
    }

    #[test]
    fn a_frame_taller_than_the_terminal_ends_with_an_ellipsis_until_the_display_ends() {
        let (live, capture) = live(true, false, 3);
        live.update("a\nb\nc\nd");
        live.start();
        live.stop();
        assert_eq!(
            capture.text(),
            "\x1b[?25la\nb\n        ...         \r\x1b[2K\x1b[1A\x1b[2K\x1b[1A\x1b[2Ka\nb\nc\nd\n\x1b[?25h"
        );
    }

    #[test]
    fn the_last_handle_ends_the_display() {
        let (live, capture) = live(true, false, 10);
        live.start();
        let other = live.clone();
        drop(live);
        assert!(!capture.text().ends_with("\x1b[?25h"));
        drop(other);
        assert!(capture.text().ends_with("\x1b[?25h"));
    }

    #[test]
    fn a_disabled_display_writes_nothing() {
        let capture = Capture::default();
        let live = Live::builder()
            .renderable("x")
            .console(console(10))
            .interactive(true)
            .disable(true)
            .writer(capture.clone())
            .build();
        live.start();
        live.refresh();
        live.stop();
        assert_eq!(capture.text(), "");
    }
}
