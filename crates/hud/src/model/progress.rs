use super::color::Color;
use super::style::Style;

/// Text built from a template, one line per task.
///
/// The template reads `{task.description}`, `{task.completed}` and `{task.total}` and is then
/// read as markup, so `"[bold]{task.description}[/]"` works. Anything else in braces is kept
/// as it is.
///
/// ```
/// use hud::TextColumn;
///
/// let column = TextColumn::new("[bold]{task.description}[/] ({task.completed} of {task.total})");
/// # let _ = column;
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TextColumn {
    pub(crate) template: String,
    pub(crate) style: Style,
}

impl TextColumn {
    /// A column that prints `template` for every task.
    pub fn new(template: impl Into<String>) -> TextColumn {
        TextColumn {
            template: template.into(),
            style: Style::new(),
        }
    }

    /// The style every cell starts from; markup in the template is applied over it.
    #[must_use]
    pub fn style(mut self, style: Style) -> TextColumn {
        self.style = style;
        self
    }
}

/// A bar that fills as the task completes, drawn with half-cell resolution.
///
/// ```
/// use hud::BarColumn;
///
/// let column = BarColumn::new().bar_width(30);
/// # let _ = column;
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BarColumn {
    pub(crate) bar_width: Option<usize>,
    pub(crate) back: Style,
    pub(crate) complete: Style,
    pub(crate) finished: Style,
}

impl BarColumn {
    /// A 40 cell bar in Rich's colors: grey track, pink while running, green when finished.
    pub fn new() -> BarColumn {
        BarColumn {
            bar_width: Some(40),
            back: Style::new().color(Color::indexed(237)),
            complete: Style::new().color(Color::rgb(249, 38, 114)),
            finished: Style::new().color(Color::rgb(114, 156, 31)),
        }
    }

    /// The width of the bar in cells (at least 1); the bar shrinks when the line is too narrow.
    #[must_use]
    pub fn bar_width(mut self, width: usize) -> BarColumn {
        self.bar_width = Some(width.max(1));
        self
    }

    /// Lets the bar take whatever width the other columns leave.
    #[must_use]
    pub fn full_width(mut self) -> BarColumn {
        self.bar_width = None;
        self
    }

    /// The style of the part that is not complete yet.
    #[must_use]
    pub fn style(mut self, style: Style) -> BarColumn {
        self.back = style;
        self
    }

    /// The style of the complete part while the task runs.
    #[must_use]
    pub fn complete_style(mut self, style: Style) -> BarColumn {
        self.complete = style;
        self
    }

    /// The style of the bar once the task is finished.
    #[must_use]
    pub fn finished_style(mut self, style: Style) -> BarColumn {
        self.finished = style;
        self
    }
}

impl Default for BarColumn {
    fn default() -> BarColumn {
        BarColumn::new()
    }
}

/// The percentage of the task that is complete, as ` 75%`.
///
/// ```
/// use hud::TaskProgressColumn;
///
/// let column = TaskProgressColumn::new();
/// # let _ = column;
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TaskProgressColumn {
    pub(crate) style: Style,
}

impl TaskProgressColumn {
    /// A magenta percentage column.
    pub fn new() -> TaskProgressColumn {
        TaskProgressColumn {
            style: Style::new().color(Color::MAGENTA),
        }
    }

    /// The style of the percentage.
    #[must_use]
    pub fn style(mut self, style: Style) -> TaskProgressColumn {
        self.style = style;
        self
    }
}

impl Default for TaskProgressColumn {
    fn default() -> TaskProgressColumn {
        TaskProgressColumn::new()
    }
}

/// Completed over total, as ` 75/100`, padded so the width does not change as the count grows.
///
/// ```
/// use hud::MofNCompleteColumn;
///
/// let column = MofNCompleteColumn::new().separator(" of ");
/// # let _ = column;
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MofNCompleteColumn {
    pub(crate) separator: String,
    pub(crate) style: Style,
}

impl MofNCompleteColumn {
    /// A green `completed/total` column.
    pub fn new() -> MofNCompleteColumn {
        MofNCompleteColumn {
            separator: "/".to_string(),
            style: Style::new().color(Color::GREEN),
        }
    }

    /// The text between the two numbers (default `/`).
    #[must_use]
    pub fn separator(mut self, separator: impl Into<String>) -> MofNCompleteColumn {
        self.separator = separator.into();
        self
    }

    /// The style of the counts.
    #[must_use]
    pub fn style(mut self, style: Style) -> MofNCompleteColumn {
        self.style = style;
        self
    }
}

impl Default for MofNCompleteColumn {
    fn default() -> MofNCompleteColumn {
        MofNCompleteColumn::new()
    }
}

/// The time since the task started, as `0:01:05`; it stops when the task finishes.
///
/// ```
/// use hud::TimeElapsedColumn;
///
/// let column = TimeElapsedColumn::new();
/// # let _ = column;
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TimeElapsedColumn {
    pub(crate) style: Style,
}

impl TimeElapsedColumn {
    /// A yellow elapsed time column.
    pub fn new() -> TimeElapsedColumn {
        TimeElapsedColumn {
            style: Style::new().color(Color::YELLOW),
        }
    }

    /// The style of the time.
    #[must_use]
    pub fn style(mut self, style: Style) -> TimeElapsedColumn {
        self.style = style;
        self
    }
}

impl Default for TimeElapsedColumn {
    fn default() -> TimeElapsedColumn {
        TimeElapsedColumn::new()
    }
}

/// The estimated time left, as `0:00:42`, from the speed of the last 30 seconds.
///
/// ```
/// use hud::TimeRemainingColumn;
///
/// let column = TimeRemainingColumn::new().compact(true);
/// # let _ = column;
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TimeRemainingColumn {
    pub(crate) compact: bool,
    pub(crate) elapsed_when_finished: bool,
    pub(crate) style: Style,
    pub(crate) elapsed_style: Style,
}

impl TimeRemainingColumn {
    /// A cyan time remaining column.
    pub fn new() -> TimeRemainingColumn {
        TimeRemainingColumn {
            compact: false,
            elapsed_when_finished: false,
            style: Style::new().color(Color::CYAN),
            elapsed_style: Style::new().color(Color::YELLOW),
        }
    }

    /// Shows `MM:SS` when less than an hour is left.
    #[must_use]
    pub fn compact(mut self, compact: bool) -> TimeRemainingColumn {
        self.compact = compact;
        self
    }

    /// Shows the time the task took once it is finished.
    #[must_use]
    pub fn elapsed_when_finished(mut self, elapsed_when_finished: bool) -> TimeRemainingColumn {
        self.elapsed_when_finished = elapsed_when_finished;
        self
    }

    /// The style of the estimate.
    #[must_use]
    pub fn style(mut self, style: Style) -> TimeRemainingColumn {
        self.style = style;
        self
    }
}

impl Default for TimeRemainingColumn {
    fn default() -> TimeRemainingColumn {
        TimeRemainingColumn::new()
    }
}

/// A spinner that turns while the task runs and is replaced by a blank when it finishes.
///
/// ```
/// use hud::SpinnerColumn;
///
/// let column = SpinnerColumn::new().finished_text("done");
/// # let _ = column;
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct SpinnerColumn {
    pub(crate) style: Style,
    pub(crate) speed: f64,
    pub(crate) finished_text: String,
}

impl SpinnerColumn {
    /// A green dots spinner that shows a blank once the task is finished.
    pub fn new() -> SpinnerColumn {
        SpinnerColumn {
            style: Style::new().color(Color::GREEN),
            speed: 1.0,
            finished_text: " ".to_string(),
        }
    }

    /// The style of the spinner.
    #[must_use]
    pub fn style(mut self, style: Style) -> SpinnerColumn {
        self.style = style;
        self
    }

    /// Turns faster (above 1) or slower (below 1).
    #[must_use]
    pub fn speed(mut self, speed: f64) -> SpinnerColumn {
        self.speed = speed;
        self
    }

    /// The markup shown in place of the spinner once the task is finished.
    #[must_use]
    pub fn finished_text(mut self, text: impl Into<String>) -> SpinnerColumn {
        self.finished_text = text.into();
        self
    }
}

impl Default for SpinnerColumn {
    fn default() -> SpinnerColumn {
        SpinnerColumn::new()
    }
}

/// One column of a progress display; build them with the `*Column` types and pass them to
/// [`ProgressBuilder::column`](crate::ProgressBuilder::column).
#[derive(Clone, Debug, PartialEq)]
pub enum ProgressColumn {
    /// See [`TextColumn`].
    Text(TextColumn),
    /// See [`BarColumn`].
    Bar(BarColumn),
    /// See [`TaskProgressColumn`].
    TaskProgress(TaskProgressColumn),
    /// See [`MofNCompleteColumn`].
    MofNComplete(MofNCompleteColumn),
    /// See [`TimeElapsedColumn`].
    TimeElapsed(TimeElapsedColumn),
    /// See [`TimeRemainingColumn`].
    TimeRemaining(TimeRemainingColumn),
    /// See [`SpinnerColumn`].
    Spinner(SpinnerColumn),
}

macro_rules! column_from {
    ($($ty:ident => $variant:ident),* $(,)?) => {
        $(impl From<$ty> for ProgressColumn {
            fn from(column: $ty) -> ProgressColumn {
                ProgressColumn::$variant(column)
            }
        })*
    };
}

column_from! {
    TextColumn => Text,
    BarColumn => Bar,
    TaskProgressColumn => TaskProgress,
    MofNCompleteColumn => MofNComplete,
    TimeElapsedColumn => TimeElapsed,
    TimeRemainingColumn => TimeRemaining,
    SpinnerColumn => Spinner,
}

/// What a column needs to know about one task at one moment: the numbers and the times.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct TaskSnapshot {
    pub(crate) description: String,
    pub(crate) total: u64,
    pub(crate) completed: u64,
    /// Seconds since the task started, `None` before it has.
    pub(crate) elapsed: Option<f64>,
    /// How long the task took, once it finished.
    pub(crate) finished_time: Option<f64>,
    /// Estimated seconds left, `None` without enough samples.
    pub(crate) time_remaining: Option<f64>,
}

impl TaskSnapshot {
    pub(crate) fn finished(&self) -> bool {
        self.finished_time.is_some()
    }
}
