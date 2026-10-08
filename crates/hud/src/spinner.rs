//! Spinner: one frame of a named animation next to a message, turning with a clock.
//!
//! This is composition, like [`Live`](crate::Live): the animation arithmetic is pure
//! (`model::Animation`), the text is put together by a pure service, and this file holds the
//! clock and the state that clones share.

use std::fmt;
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use std::time::Instant;

use crate::model::{
    Animation, Body, Measure, Node, Renderable, Segment, Style, Text, UnknownSpinner,
    default_spinner, find_spinner, spinner_names,
};
use crate::services::frame::markup_text;
use crate::services::node::text_node;
use crate::services::spinner::assemble;

pub(crate) type Clock = Arc<dyn Fn() -> f64 + Send + Sync>;

fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(PoisonError::into_inner)
}

/// A clock that counts seconds from now, on the monotonic clock of the machine.
pub(crate) fn monotonic() -> Clock {
    let origin = Instant::now();
    Arc::new(move || origin.elapsed().as_secs_f64())
}

struct State {
    animation: Animation,
    message: Text,
    style: Style,
}

/// One frame of a named animation, followed by a message: `⠋ loading`.
///
/// The frame shown depends on a clock (seconds on the monotonic clock by default), so printing
/// the same spinner later shows a later frame; the first time it is drawn fixes where the
/// animation starts. [`Spinner::names`] lists the 73 animations; `"dots"` is the usual one. A
/// spinner is a cheap handle: clones share one animation, so a copy given to a
/// [`Live`](crate::Live) turns while the original changes its message with [`Spinner::update`].
/// A [`Status`](crate::Status) is a spinner on a live display.
///
/// The message is read as markup. In [`Format::Json`](crate::Format::Json) a spinner is the text
/// of its message, with no frame and no time, so a document is the same whatever the clock says.
///
/// ```
/// use hud::{Console, Spinner};
///
/// let spinner = Spinner::new("dots").text("[bold]loading[/]").clock(|| 0.0);
/// let console = Console::builder().width(20).plain().build();
/// assert_eq!(console.render_to_plain(&spinner), "⠋ loading\n");
/// ```
#[derive(Clone)]
pub struct Spinner {
    state: Arc<Mutex<State>>,
    clock: Clock,
}

impl Spinner {
    /// A spinner with the animation called `name`, no message and no style, turning at normal
    /// speed. A name that is not one of [`Spinner::names`] gives `"dots"`; use
    /// [`Spinner::try_new`] to find out.
    ///
    /// ```
    /// use hud::Spinner;
    ///
    /// assert_eq!(Spinner::new("line").name(), "line");
    /// assert_eq!(Spinner::new("no such animation").name(), "dots");
    /// ```
    pub fn new(name: &str) -> Spinner {
        let data = find_spinner(name).unwrap_or_else(default_spinner);
        Spinner::from_animation(Animation::new(data, 1.0))
    }

    /// Like [`Spinner::new`], but a name that is not an animation is an error.
    ///
    /// ```
    /// use hud::Spinner;
    ///
    /// assert!(Spinner::try_new("earth").is_ok());
    /// assert_eq!(Spinner::try_new("nope").unwrap_err().name(), "nope");
    /// ```
    pub fn try_new(name: &str) -> Result<Spinner, UnknownSpinner> {
        match find_spinner(name) {
            Some(data) => Ok(Spinner::from_animation(Animation::new(data, 1.0))),
            None => Err(UnknownSpinner {
                name: name.to_string(),
            }),
        }
    }

    fn from_animation(animation: Animation) -> Spinner {
        Spinner {
            state: Arc::new(Mutex::new(State {
                animation,
                message: Text::new(""),
                style: Style::new(),
            })),
            clock: monotonic(),
        }
    }

    /// The names of every animation, in alphabetical order.
    ///
    /// ```
    /// assert!(hud::Spinner::names().any(|name| name == "bouncingBall"));
    /// ```
    pub fn names() -> impl Iterator<Item = &'static str> {
        spinner_names()
    }

    /// The name of this spinner's animation.
    pub fn name(&self) -> &'static str {
        lock(&self.state).animation.name()
    }

    /// The message after the frame, read as markup.
    #[must_use]
    pub fn text(self, markup: impl AsRef<str>) -> Spinner {
        lock(&self.state).message = markup_text(markup.as_ref());
        self
    }

    /// The style of the frame.
    #[must_use]
    pub fn style(self, style: Style) -> Spinner {
        lock(&self.state).style = style;
        self
    }

    /// Turns faster (above 1) or slower (below 1). Zero changes nothing.
    #[must_use]
    pub fn speed(self, speed: f64) -> Spinner {
        {
            let mut state = lock(&self.state);
            state.animation = state.animation.clone().with_speed(speed);
        }
        self
    }

    /// The same spinner on a clock that is already shared.
    pub(crate) fn with_clock(mut self, clock: Clock) -> Spinner {
        self.clock = clock;
        self
    }

    /// Replaces the clock, which returns seconds, so frames are the same on every run.
    #[must_use]
    pub fn clock(mut self, clock: impl Fn() -> f64 + Send + Sync + 'static) -> Spinner {
        self.clock = Arc::new(clock);
        self
    }

    /// Changes the message, the style or the speed while the spinner turns. A new speed takes
    /// effect at the next frame and keeps the frame that was showing. The change is made when
    /// the returned value is dropped, so `spinner.update().text("done");` is one statement.
    ///
    /// ```
    /// use hud::{Console, Spinner};
    ///
    /// let spinner = Spinner::new("dots").text("loading").clock(|| 0.0);
    /// spinner.update().text("saving");
    /// let console = Console::builder().width(20).plain().build();
    /// assert_eq!(console.render_to_plain(&spinner), "⠋ saving\n");
    /// ```
    pub fn update(&self) -> SpinnerUpdate {
        SpinnerUpdate {
            spinner: self.clone(),
            text: None,
            style: None,
            speed: None,
        }
    }

    /// The text of the frame at `time` on this spinner's clock.
    pub(crate) fn text_at(&self, time: f64) -> Text {
        let mut state = lock(&self.state);
        let frame = state.animation.frame(time);
        assemble(frame, &state.style, &state.message)
    }
}

impl fmt::Debug for Spinner {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Spinner")
            .field("name", &self.name())
            .finish_non_exhaustive()
    }
}

impl Renderable for Spinner {
    fn render(&self, width: usize) -> Vec<Segment> {
        self.text_at((self.clock)()).render(width)
    }

    /// Like Rich, measuring asks for the frame at time zero, which also fixes the start of an
    /// animation that has not been drawn yet.
    fn measure(&self, max_width: usize) -> Measure {
        self.text_at(0.0).measure(max_width)
    }

    fn node(&self) -> Node {
        text_node(&lock(&self.state).message)
    }
}

impl From<Spinner> for Body {
    fn from(spinner: Spinner) -> Body {
        Body::new(spinner)
    }
}

/// A change to a [`Spinner`], made when this value is dropped; made by [`Spinner::update`].
pub struct SpinnerUpdate {
    spinner: Spinner,
    text: Option<String>,
    style: Option<Style>,
    speed: Option<f64>,
}

impl SpinnerUpdate {
    /// The new message, read as markup. An empty message changes nothing.
    pub fn text(mut self, markup: impl Into<String>) -> SpinnerUpdate {
        self.text = Some(markup.into());
        self
    }

    /// The new style of the frame. A style with no attribute changes nothing.
    pub fn style(mut self, style: Style) -> SpinnerUpdate {
        self.style = Some(style);
        self
    }

    /// The new speed, from the next frame on. Zero changes nothing.
    pub fn speed(mut self, speed: f64) -> SpinnerUpdate {
        self.speed = Some(speed);
        self
    }
}

impl Drop for SpinnerUpdate {
    fn drop(&mut self) {
        let mut state = lock(&self.spinner.state);
        if let Some(text) = self.text.take().filter(|text| !text.is_empty()) {
            state.message = markup_text(&text);
        }
        if let Some(style) = self.style.take().filter(|style| !style.is_null()) {
            state.style = style;
        }
        if let Some(speed) = self.speed.take() {
            state.animation.set_speed(speed);
        }
    }
}

impl fmt::Debug for SpinnerUpdate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SpinnerUpdate").finish_non_exhaustive()
    }
}
