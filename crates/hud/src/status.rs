//! Status: a spinner on a transient live display.
//!
//! Composition over [`Live`](crate::Live): the redraw protocol is the one `Live` and
//! [`Progress`](crate::Progress) share; this file keeps the message, the style and the speed
//! that Rich's `Status.update` carries from one change to the next.

use std::fmt;
use std::io::Write;
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use crate::console::Console;
use crate::live::Live;
use crate::model::{Color, Measure, Node, Renderable, Segment, Style};
use crate::spinner::{Clock, Spinner, monotonic};

fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(PoisonError::into_inner)
}

struct Inner {
    text: String,
    style: Style,
    speed: f64,
    spinner: Spinner,
}

struct Shared {
    inner: Mutex<Inner>,
    clock: Clock,
}

/// A message with a spinner next to it, shown while something runs and gone when it ends.
///
/// On a terminal the status is drawn in place and a background thread turns the spinner 12.5
/// times a second; when it ends the whole display is erased (a status is transient). A stream
/// that is not a terminal gets nothing. [`Status::update`] changes the message, the spinner, its
/// style and its speed while it runs. A `Status` is a cheap handle: clone it, move it to a
/// thread, call every method through a shared reference. The display ends on [`Status::stop`] or
/// when the last handle is dropped.
///
/// ```no_run
/// use hud::Status;
///
/// let status = Status::new("[bold]fetching[/] the index");
/// status.start();
/// // ... work ...
/// status.update().text("unpacking").spinner("line");
/// // ... more work ...
/// status.stop();
/// ```
#[derive(Clone)]
pub struct Status {
    live: Live,
    shared: Arc<Shared>,
}

impl Status {
    /// A status with `text` (read as markup) and the `dots` spinner in green, on standard
    /// output, not started yet.
    pub fn new(text: impl AsRef<str>) -> Status {
        Status::builder().text(text).build()
    }

    /// A builder to choose the console, the spinner, its style and speed and the refresh.
    pub fn builder() -> StatusBuilder {
        StatusBuilder::new()
    }

    /// Starts the display: the cursor is hidden and the spinner begins to turn. Calling it
    /// again does nothing.
    pub fn start(&self) {
        self.live.start();
    }

    /// Ends the display: the last frame is drawn and erased, the cursor comes back and nothing
    /// is drawn afterwards. Calling it again does nothing.
    pub fn stop(&self) {
        self.live.stop();
    }

    /// Draws the status now, instead of at the next automatic refresh.
    pub fn refresh(&self) {
        self.live.refresh();
    }

    /// Whether the display has been started and not stopped.
    pub fn is_started(&self) -> bool {
        self.live.is_started()
    }

    /// Changes the message, the spinner, its style or its speed while the status runs. A new
    /// spinner replaces the old one and is drawn at once; anything else changes the spinner that
    /// is turning, and shows at the next frame. The change is made when the returned value is
    /// dropped, so `status.update().text("done");` is one statement.
    pub fn update(&self) -> StatusUpdate {
        StatusUpdate {
            status: self.clone(),
            text: None,
            spinner: None,
            style: None,
            speed: None,
        }
    }

    /// The spinner that is turning now.
    pub fn spinner(&self) -> Spinner {
        lock(&self.shared.inner).spinner.clone()
    }
}

impl fmt::Debug for Status {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Status").finish_non_exhaustive()
    }
}

/// A status prints as its spinner, like Rich's `Status.__rich__`.
impl Renderable for Status {
    fn render(&self, width: usize) -> Vec<Segment> {
        self.spinner().render(width)
    }

    fn measure(&self, max_width: usize) -> Measure {
        self.spinner().measure(max_width)
    }

    fn node(&self) -> Node {
        self.spinner().node()
    }
}

/// A change to a [`Status`], made when this value is dropped; made by [`Status::update`].
pub struct StatusUpdate {
    status: Status,
    text: Option<String>,
    spinner: Option<String>,
    style: Option<Style>,
    speed: Option<f64>,
}

impl StatusUpdate {
    /// The new message, read as markup.
    pub fn text(mut self, markup: impl Into<String>) -> StatusUpdate {
        self.text = Some(markup.into());
        self
    }

    /// The name of a new spinner (one of [`Spinner::names`]); an unknown name gives `dots`.
    pub fn spinner(mut self, name: impl Into<String>) -> StatusUpdate {
        self.spinner = Some(name.into());
        self
    }

    /// The new style of the spinner.
    pub fn spinner_style(mut self, style: Style) -> StatusUpdate {
        self.style = Some(style);
        self
    }

    /// The new speed of the spinner.
    pub fn speed(mut self, speed: f64) -> StatusUpdate {
        self.speed = Some(speed);
        self
    }
}

impl Drop for StatusUpdate {
    fn drop(&mut self) {
        let shared = &self.status.shared;
        let mut inner = lock(&shared.inner);
        if let Some(text) = self.text.take() {
            inner.text = text;
        }
        if let Some(style) = self.style.take() {
            inner.style = style;
        }
        if let Some(speed) = self.speed.take() {
            inner.speed = speed;
        }
        if let Some(name) = self.spinner.take() {
            let spinner = Spinner::new(&name)
                .text(&inner.text)
                .style(inner.style.clone())
                .speed(inner.speed)
                .with_clock(Arc::clone(&shared.clock));
            inner.spinner = spinner.clone();
            drop(inner);
            self.status.live.update(spinner);
            self.status.refresh();
        } else {
            inner
                .spinner
                .update()
                .text(inner.text.clone())
                .style(inner.style.clone())
                .speed(inner.speed);
        }
    }
}

impl fmt::Debug for StatusUpdate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("StatusUpdate").finish_non_exhaustive()
    }
}

/// Builds a [`Status`]; made by [`Status::builder`].
///
/// ```
/// use hud::{Color, Status, Style};
///
/// let status = Status::builder()
///     .text("[bold]building[/]")
///     .spinner("moon")
///     .spinner_style(Style::new().color(Color::YELLOW))
///     .refresh_per_second(20.0)
///     .disable(true)
///     .build();
/// status.start();
/// status.stop();
/// ```
pub struct StatusBuilder {
    text: String,
    console: Console,
    spinner: String,
    style: Style,
    speed: f64,
    refresh_per_second: f64,
    auto_refresh: bool,
    clock: Option<Clock>,
    writer: Option<Box<dyn Write + Send>>,
    interactive: Option<bool>,
    disable: bool,
}

impl StatusBuilder {
    fn new() -> StatusBuilder {
        StatusBuilder {
            text: String::new(),
            console: Console::stdout(),
            spinner: "dots".to_string(),
            style: Style::new().color(Color::GREEN),
            speed: 1.0,
            refresh_per_second: 12.5,
            auto_refresh: true,
            clock: None,
            writer: None,
            interactive: None,
            disable: false,
        }
    }

    /// The message, read as markup (default: none).
    #[must_use]
    pub fn text(mut self, markup: impl AsRef<str>) -> StatusBuilder {
        self.text = markup.as_ref().to_string();
        self
    }

    /// The console the status draws on (default: standard output).
    #[must_use]
    pub fn console(mut self, console: Console) -> StatusBuilder {
        self.console = console;
        self
    }

    /// The name of the spinner, one of [`Spinner::names`] (default `dots`; an unknown name
    /// gives `dots`).
    #[must_use]
    pub fn spinner(mut self, name: impl Into<String>) -> StatusBuilder {
        self.spinner = name.into();
        self
    }

    /// The style of the spinner (default: green).
    #[must_use]
    pub fn spinner_style(mut self, style: Style) -> StatusBuilder {
        self.style = style;
        self
    }

    /// Turns faster (above 1) or slower (below 1).
    #[must_use]
    pub fn speed(mut self, speed: f64) -> StatusBuilder {
        self.speed = speed;
        self
    }

    /// How often the background thread redraws (default 12.5).
    #[must_use]
    pub fn refresh_per_second(mut self, per_second: f64) -> StatusBuilder {
        self.refresh_per_second = per_second;
        self
    }

    /// Whether a background thread redraws the status by itself on a terminal (default: yes).
    /// Without it the status draws only when it ends and when its spinner is replaced.
    #[must_use]
    pub fn auto_refresh(mut self, auto_refresh: bool) -> StatusBuilder {
        self.auto_refresh = auto_refresh;
        self
    }

    /// Replaces the clock, which returns seconds, so frames are the same on every run.
    #[must_use]
    pub fn clock(mut self, clock: impl Fn() -> f64 + Send + Sync + 'static) -> StatusBuilder {
        self.clock = Some(Arc::new(clock));
        self
    }

    /// Writes to `writer` instead of the console's stream.
    #[must_use]
    pub fn writer(mut self, writer: impl Write + Send + 'static) -> StatusBuilder {
        self.writer = Some(Box::new(writer));
        self
    }

    /// Forces redrawing in place (`true`) or writing nothing (`false`), in place of what the
    /// console reports.
    #[must_use]
    pub fn interactive(mut self, interactive: bool) -> StatusBuilder {
        self.interactive = Some(interactive);
        self
    }

    /// Draws nothing at all.
    #[must_use]
    pub fn disable(mut self, disable: bool) -> StatusBuilder {
        self.disable = disable;
        self
    }

    /// Finishes the status.
    pub fn build(self) -> Status {
        let clock = self.clock.unwrap_or_else(monotonic);
        let spinner = Spinner::new(&self.spinner)
            .text(&self.text)
            .style(self.style.clone())
            .speed(self.speed)
            .with_clock(Arc::clone(&clock));
        let mut live = Live::builder()
            .console(self.console)
            .transient(true)
            .auto_refresh(self.auto_refresh)
            .refresh_per_second(self.refresh_per_second)
            .disable(self.disable);
        if let Some(writer) = self.writer {
            live = live.writer(WriterBox(writer));
        }
        if let Some(interactive) = self.interactive {
            live = live.interactive(interactive);
        }
        let live = live.build();
        live.update(spinner.clone());
        Status {
            live,
            shared: Arc::new(Shared {
                inner: Mutex::new(Inner {
                    text: self.text,
                    style: self.style,
                    speed: self.speed,
                    spinner,
                }),
                clock,
            }),
        }
    }
}

impl fmt::Debug for StatusBuilder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("StatusBuilder").finish_non_exhaustive()
    }
}

/// A boxed writer is itself a writer, so it can go through [`LiveBuilder::writer`](crate::LiveBuilder::writer).
struct WriterBox(Box<dyn Write + Send>);

impl Write for WriterBox {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.0.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.0.flush()
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

    fn clock(now: &Arc<Mutex<f64>>) -> impl Fn() -> f64 + Send + Sync + 'static {
        let now = Arc::clone(now);
        move || *lock(&now)
    }

    fn console() -> Console {
        Console::builder()
            .width(20)
            .height(10)
            .color_system(ColorSystem::None)
            .attributes(false)
            .build()
    }

    fn status(interactive: bool) -> (Status, Capture, Arc<Mutex<f64>>) {
        let capture = Capture::default();
        let now = Arc::new(Mutex::new(0.0));
        let status = Status::builder()
            .text("working")
            .console(console())
            .interactive(interactive)
            .auto_refresh(false)
            .clock(clock(&now))
            .writer(capture.clone())
            .build();
        (status, capture, now)
    }

    #[test]
    fn starting_draws_nothing_until_the_first_refresh_and_the_end_erases_the_display() {
        let (status, capture, now) = status(true);
        status.start();
        assert_eq!(capture.text(), "\x1b[?25l");
        *lock(&now) = 0.16;
        status.stop();
        assert_eq!(
            capture.text(),
            "\x1b[?25l⠋ working\n\x1b[?25h\r\x1b[1A\x1b[2K"
        );
    }

    #[test]
    fn the_frame_follows_the_clock() {
        let (status, capture, now) = status(true);
        status.start();
        status.refresh();
        *lock(&now) = 0.08;
        status.refresh();
        *lock(&now) = 0.16;
        status.stop();
        assert!(capture.text().contains("⠋ working"));
        assert!(capture.text().contains("⠙ working"));
        assert!(capture.text().contains("⠹ working"));
    }

    #[test]
    fn a_new_spinner_is_drawn_at_once_and_a_new_text_waits_for_the_next_frame() {
        let (status, capture, now) = status(true);
        status.start();
        *lock(&now) = 0.08;
        status.update().spinner("line").text("copying");
        assert!(capture.text().ends_with("- copying"));
        status.update().text("copied");
        assert!(capture.text().ends_with("- copying"));
        *lock(&now) = 0.25;
        status.refresh();
        assert!(capture.text().ends_with("\\ copied"));
    }

    #[test]
    fn a_stream_that_is_not_a_terminal_gets_nothing() {
        let (status, capture, now) = status(false);
        status.start();
        *lock(&now) = 1.0;
        status.update().spinner("arc").text("x");
        status.stop();
        assert_eq!(capture.text(), "");
    }

    #[test]
    fn a_status_prints_as_its_spinner_and_describes_itself_as_its_message() {
        let (status, _, _) = status(true);
        assert_eq!(console().render_to_plain(&status), "⠋ working\n");
        assert_eq!(status.node(), crate::Node::text("working"));
    }
}
