//! Live progress: tasks, the handles that advance them, and in-place redrawing.
//!
//! This is composition, like [`Console`]: it connects the pure progress service to the stream.

use std::collections::VecDeque;
use std::fmt;
use std::io::Write;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Condvar, Mutex, MutexGuard, PoisonError, RwLock};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use crate::console::{Console, capabilities};
use crate::integrations;
use crate::model::{
    BarColumn, Capabilities, ColorSystem, Justify, ProgressColumn, Renderable, Segment, Stream,
    Style, TaskProgressColumn, TaskSnapshot, Text, TextColumn, TimeRemainingColumn,
};
use crate::services::progress::{FastPlan, Frame, render_lines};
use crate::services::render::{crop_lines, render_text_ending, to_ansi};

type Clock = Arc<dyn Fn() -> f64 + Send + Sync>;

/// Seconds of history the speed estimate looks at, as in Rich.
const SPEED_PERIOD: f64 = 30.0;
/// The most samples kept per task.
const MAX_SAMPLES: usize = 1000;

fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(PoisonError::into_inner)
}

#[derive(Default)]
struct Times {
    start: Option<f64>,
    finished: Option<f64>,
    samples: VecDeque<(f64, u64)>,
}

impl Times {
    fn elapsed(&self, now: f64) -> Option<f64> {
        self.start.map(|start| now - start)
    }

    fn prune(&mut self, now: f64, period: f64) {
        let oldest = now - period;
        while self.samples.front().is_some_and(|&(at, _)| at < oldest) {
            self.samples.pop_front();
        }
        while self.samples.len() >= MAX_SAMPLES {
            self.samples.pop_front();
        }
    }

    fn speed(&self) -> Option<f64> {
        self.start?;
        let (first, last) = (self.samples.front()?, self.samples.back()?);
        let span = last.0 - first.0;
        if span == 0.0 {
            return None;
        }
        let done: u64 = self.samples.iter().skip(1).map(|&(_, n)| n).sum();
        Some(done as f64 / span)
    }
}

struct Shared {
    clock: Clock,
    track_speed: bool,
    period: f64,
    /// Bumped whenever a description, a total or the set of tasks changes, so cached layouts
    /// can tell they are stale.
    layout_version: AtomicU64,
}

struct TaskCell {
    shared: Arc<Shared>,
    description: Mutex<String>,
    total: AtomicU64,
    completed: AtomicU64,
    finished: AtomicBool,
    times: Mutex<Times>,
}

impl TaskCell {
    fn new(shared: Arc<Shared>, description: String, total: u64) -> TaskCell {
        let now = (shared.clock)();
        TaskCell {
            shared,
            description: Mutex::new(description),
            total: AtomicU64::new(total),
            completed: AtomicU64::new(0),
            finished: AtomicBool::new(false),
            times: Mutex::new(Times {
                start: Some(now),
                ..Times::default()
            }),
        }
    }

    fn sample(&self, amount: u64) {
        let now = (self.shared.clock)();
        let mut times = lock(&self.times);
        times.prune(now, self.shared.period);
        times.samples.push_back((now, amount));
    }

    fn check_finished(&self, completed: u64) {
        if completed >= self.total.load(Ordering::Relaxed) && !self.finished.load(Ordering::Acquire)
        {
            let now = (self.shared.clock)();
            let mut times = lock(&self.times);
            if times.finished.is_none() {
                times.finished = times.elapsed(now);
            }
            self.finished
                .store(times.finished.is_some(), Ordering::Release);
        }
    }

    fn advance(&self, amount: u64) {
        let after = self
            .completed
            .fetch_add(amount, Ordering::Relaxed)
            .saturating_add(amount);
        if self.shared.track_speed {
            self.sample(amount);
        }
        self.check_finished(after);
    }

    fn set_completed(&self, completed: u64) {
        let before = self.completed.swap(completed, Ordering::Relaxed);
        if self.shared.track_speed && completed > before {
            self.sample(completed - before);
        }
        self.check_finished(completed);
    }

    fn set_total(&self, total: u64) {
        if self.total.swap(total, Ordering::Relaxed) != total {
            self.shared.layout_version.fetch_add(1, Ordering::Relaxed);
            let mut times = lock(&self.times);
            times.samples.clear();
            times.finished = None;
            self.finished.store(false, Ordering::Release);
        }
        self.check_finished(self.completed.load(Ordering::Relaxed));
    }

    fn snapshot(&self, now: f64) -> TaskSnapshot {
        let total = self.total.load(Ordering::Relaxed);
        let completed = self.completed.load(Ordering::Relaxed);
        let times = lock(&self.times);
        let time_remaining = if times.finished.is_some() {
            Some(0.0)
        } else {
            times
                .speed()
                .filter(|speed| *speed != 0.0)
                .map(|speed| ((total as f64 - completed as f64) / speed).ceil())
        };
        TaskSnapshot {
            description: lock(&self.description).clone(),
            total,
            completed,
            elapsed: times.elapsed(now),
            finished_time: times.finished,
            time_remaining,
        }
    }
}

/// A handle to one task of a [`Progress`]. Cloning is cheap, every method takes `&self`, and
/// the handle can move to another thread.
///
/// ```
/// use hud::Progress;
///
/// let progress = Progress::builder().disable(true).build();
/// let task = progress.add_task("download", 10);
/// task.advance(4);
/// assert_eq!((task.completed(), task.total()), (4, 10));
/// ```
#[derive(Clone)]
pub struct Task {
    cell: Arc<TaskCell>,
}

impl Task {
    /// Moves the task forward by `amount` steps.
    pub fn advance(&self, amount: u64) {
        self.cell.advance(amount);
    }

    /// Sets how many steps are done.
    pub fn set_completed(&self, completed: u64) {
        self.cell.set_completed(completed);
    }

    /// Changes the number of steps the task has; the speed estimate starts over.
    pub fn set_total(&self, total: u64) {
        self.cell.set_total(total);
    }

    /// Changes the description, read as markup.
    pub fn set_description(&self, description: impl Into<String>) {
        *lock(&self.cell.description) = description.into();
        self.cell
            .shared
            .layout_version
            .fetch_add(1, Ordering::Relaxed);
    }

    /// Marks every step done.
    pub fn finish(&self) {
        self.cell
            .set_completed(self.cell.total.load(Ordering::Relaxed));
    }

    /// Steps done so far.
    pub fn completed(&self) -> u64 {
        self.cell.completed.load(Ordering::Relaxed)
    }

    /// Steps the task has.
    pub fn total(&self) -> u64 {
        self.cell.total.load(Ordering::Relaxed)
    }

    /// Whether every step is done.
    pub fn is_finished(&self) -> bool {
        self.cell.finished.load(Ordering::Acquire)
    }
}

impl fmt::Debug for Task {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Task")
            .field("completed", &self.completed())
            .field("total", &self.total())
            .finish()
    }
}

/// What a cached plan was built for.
#[derive(Clone, Copy, PartialEq, Eq)]
struct PlanKey {
    version: u64,
    width: usize,
    color_system: ColorSystem,
    attributes: bool,
    rows: usize,
}

struct Out {
    writer: Option<Box<dyn Write + Send>>,
    started: bool,
    stopped: bool,
    height: usize,
    origins: Vec<Option<f64>>,
    plan_key: Option<PlanKey>,
    plan: Option<FastPlan>,
    pairs: Vec<(u64, u64)>,
    buffer: String,
}

struct Core {
    shared: Arc<Shared>,
    columns: Vec<ProgressColumn>,
    tasks: RwLock<Vec<Arc<TaskCell>>>,
    console: Console,
    interactive: bool,
    transient: bool,
    disable: bool,
    out: Mutex<Out>,
}

const HIDE_CURSOR: &str = "\x1b[?25l";
const SHOW_CURSOR: &str = "\x1b[?25h";

impl Core {
    fn snapshots(&self, now: f64) -> Vec<TaskSnapshot> {
        self.tasks
            .read()
            .unwrap_or_else(PoisonError::into_inner)
            .iter()
            .map(|task| task.snapshot(now))
            .collect()
    }

    fn frame_lines(&self, out: &mut Out, caps: &Capabilities, width: usize) -> Vec<Vec<Segment>> {
        let now = (self.shared.clock)();
        let snapshots = self.snapshots(now);
        let origins: Vec<f64> = self
            .columns
            .iter()
            .zip(out.origins.iter_mut())
            .map(|(column, origin)| match column {
                ProgressColumn::Spinner(_) => *origin.get_or_insert(now),
                _ => 0.0,
            })
            .collect();
        render_lines(
            &Frame {
                columns: &self.columns,
                tasks: &snapshots,
                now,
                spinner_origins: &origins,
                draw_track: caps.color_system != ColorSystem::None,
            },
            width,
        )
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

    /// The lines as bytes, each ended by a newline.
    fn ansi(lines: Vec<Vec<Segment>>, caps: &Capabilities, width: usize) -> String {
        let mut flat = Vec::new();
        for line in lines {
            flat.extend(line);
            flat.push(Segment {
                text: "\n".to_string(),
                style: Style::new(),
            });
        }
        to_ansi(&crop_lines(flat, width), caps)
    }

    /// Cuts a frame that would not leave the terminal a line for the cursor, ending it with an
    /// ellipsis line.
    fn fit_height(lines: &mut Vec<Vec<Segment>>, caps: &Capabilities, width: usize) {
        let room = usize::from(caps.height).saturating_sub(1).max(1);
        if lines.len() > room {
            lines.truncate(room.saturating_sub(1));
            let text = Text::new("...").justify(Justify::Center);
            let mut line = render_text_ending(&text, width, "");
            line.retain(|segment| !segment.text.is_empty());
            lines.push(line);
        }
    }

    /// Moves the cursor back to the top of a frame of `height` lines and clears it.
    fn erase(buffer: &mut String, height: usize) {
        if height > 0 {
            buffer.push('\r');
            buffer.push_str("\x1b[");
            buffer.push_str(&height.to_string());
            buffer.push_str("A\x1b[J");
        }
    }

    /// Appends the current frame, a newline after every line, and returns how many lines it
    /// has. Displays of the common shape are assembled from cached pieces.
    fn frame(&self, out: &mut Out, caps: &Capabilities, buffer: &mut String) -> usize {
        let width = usize::from(caps.width);
        if let Some(height) = self.fast_frame(out, caps, width, buffer) {
            return height;
        }
        let mut lines = self.frame_lines(out, caps, width);
        Self::fit_height(&mut lines, caps, width);
        let height = lines.len();
        buffer.push_str(&Self::ansi(lines, caps, width));
        height
    }

    fn fast_frame(
        &self,
        out: &mut Out,
        caps: &Capabilities,
        width: usize,
        buffer: &mut String,
    ) -> Option<usize> {
        let tasks = self.tasks.read().unwrap_or_else(PoisonError::into_inner);
        let key = PlanKey {
            version: self.shared.layout_version.load(Ordering::Relaxed),
            width,
            color_system: caps.color_system,
            attributes: caps.attributes,
            rows: tasks.len(),
        };
        if out.plan_key != Some(key) {
            out.plan_key = Some(key);
            out.plan = None;
            let room = usize::from(caps.height).saturating_sub(1);
            if tasks.len() <= room {
                let now = (self.shared.clock)();
                let snapshots: Vec<TaskSnapshot> =
                    tasks.iter().map(|task| task.snapshot(now)).collect();
                let origins = vec![0.0; self.columns.len()];
                out.plan = FastPlan::build(
                    &Frame {
                        columns: &self.columns,
                        tasks: &snapshots,
                        now,
                        spinner_origins: &origins,
                        draw_track: caps.color_system != ColorSystem::None,
                    },
                    width,
                    caps.color_system,
                    caps.attributes,
                );
            }
        }
        let plan = out.plan.as_mut()?;
        out.pairs.clear();
        out.pairs.extend(tasks.iter().map(|task| {
            (
                task.completed.load(Ordering::Relaxed),
                task.total.load(Ordering::Relaxed),
            )
        }));
        plan.render(&out.pairs, buffer).then_some(tasks.len())
    }

    fn draw(&self, out: &mut Out) {
        if self.disable || out.stopped || !self.interactive {
            return;
        }
        let caps = *self.console.capabilities();
        let mut buffer = std::mem::take(&mut out.buffer);
        buffer.clear();
        if out.started {
            Self::erase(&mut buffer, out.height);
        } else {
            buffer.push_str(HIDE_CURSOR);
            out.started = true;
        }
        out.height = self.frame(out, &caps, &mut buffer);
        self.write(out, &buffer);
        out.buffer = buffer;
    }

    fn refresh(&self) {
        let mut out = lock(&self.out);
        self.draw(&mut out);
    }

    fn finish(&self) {
        let mut out = lock(&self.out);
        if out.stopped {
            return;
        }
        let caps = *self.console.capabilities();
        let mut buffer = String::new();
        if self.interactive {
            if !out.started
                && self
                    .tasks
                    .read()
                    .unwrap_or_else(PoisonError::into_inner)
                    .is_empty()
            {
                out.stopped = true;
                return;
            }
            if out.started {
                Self::erase(&mut buffer, out.height);
            }
            if !self.transient {
                self.frame(&mut out, &caps, &mut buffer);
            }
            if out.started {
                buffer.push_str(SHOW_CURSOR);
            }
        } else if !self.transient {
            self.frame(&mut out, &caps, &mut buffer);
        }
        out.stopped = true;
        if !self.disable {
            self.write(&mut out, &buffer);
        }
    }

    fn render_once(&self, width: usize, caps: &Capabilities) -> Vec<Segment> {
        let mut out = lock(&self.out);
        let lines = self.frame_lines(&mut out, caps, width);
        let mut flat = Vec::new();
        for line in lines {
            flat.extend(line);
            flat.push(Segment {
                text: "\n".to_string(),
                style: Style::new(),
            });
        }
        flat
    }
}

struct Refresher {
    stop: Arc<(Mutex<bool>, Condvar)>,
    handle: Option<JoinHandle<()>>,
}

impl Refresher {
    fn start(core: &Arc<Core>, interval: Duration) -> Refresher {
        let stop = Arc::new((Mutex::new(false), Condvar::new()));
        let (thread_stop, core) = (Arc::clone(&stop), Arc::clone(core));
        let handle = std::thread::Builder::new()
            .name("hud-progress".to_string())
            .spawn(move || {
                let (flag, wake) = &*thread_stop;
                loop {
                    let stopped = lock(flag);
                    let (stopped, _) = wake
                        .wait_timeout_while(stopped, interval, |stopped| !*stopped)
                        .unwrap_or_else(PoisonError::into_inner);
                    if *stopped {
                        break;
                    }
                    drop(stopped);
                    core.refresh();
                }
            })
            .ok();
        Refresher { stop, handle }
    }

    fn stop(mut self) {
        {
            let (flag, wake) = &*self.stop;
            *lock(flag) = true;
            wake.notify_all();
        }
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}

struct Inner {
    core: Arc<Core>,
    refresher: Mutex<Option<Refresher>>,
    auto_refresh: Option<Duration>,
}

impl Inner {
    fn finish(&self) {
        if let Some(refresher) = lock(&self.refresher).take() {
            refresher.stop();
        }
        self.core.finish();
    }
}

impl Drop for Inner {
    fn drop(&mut self) {
        self.finish();
    }
}

/// Progress bars for tasks that run for a while: add tasks, advance them from any thread, and
/// the display redraws in place on a terminal and prints once, at the end, anywhere else.
///
/// A `Progress` is a cheap handle: clone it, move it to threads, call every method through a
/// shared reference. The display ends, and the cursor comes back, when [`Progress::finish`] is
/// called or the last handle is dropped.
///
/// ```
/// use hud::{Console, Progress};
///
/// let progress = Progress::builder().disable(true).build();
/// let build = progress.add_task("build", 4);
/// build.advance(3);
/// let console = Console::builder().width(60).plain().build();
/// assert!(console.render_to_plain(&progress).contains("75%"));
/// ```
#[derive(Clone)]
pub struct Progress {
    inner: Arc<Inner>,
}

impl Progress {
    /// A progress display with Rich's default columns: description, bar, percentage and time
    /// remaining, on standard output.
    pub fn new() -> Progress {
        Progress::builder().build()
    }

    /// A builder to choose the columns, the console and how the display behaves.
    pub fn builder() -> ProgressBuilder {
        ProgressBuilder::new()
    }

    /// Adds a task with `total` steps and starts its clock. `description` is read as markup.
    pub fn add_task(&self, description: impl Into<String>, total: u64) -> Task {
        let cell = Arc::new(TaskCell::new(
            Arc::clone(&self.inner.core.shared),
            description.into(),
            total,
        ));
        self.inner
            .core
            .tasks
            .write()
            .unwrap_or_else(PoisonError::into_inner)
            .push(Arc::clone(&cell));
        self.inner
            .core
            .shared
            .layout_version
            .fetch_add(1, Ordering::Relaxed);
        if let Some(interval) = self.inner.auto_refresh {
            let mut refresher = lock(&self.inner.refresher);
            if refresher.is_none() {
                *refresher = Some(Refresher::start(&self.inner.core, interval));
            }
        }
        self.inner.core.refresh();
        Task { cell }
    }

    /// Moves `task` forward by `amount` steps.
    pub fn advance(&self, task: &Task, amount: u64) {
        task.advance(amount);
    }

    /// Sets how many steps of `task` are done.
    pub fn update(&self, task: &Task, completed: u64) {
        task.set_completed(completed);
    }

    /// Draws the display now.
    pub fn refresh(&self) {
        self.inner.core.refresh();
    }

    /// Ends the display: the last frame stays on screen (or goes away, for a transient
    /// display), the cursor comes back, and nothing is drawn afterwards. Calling it again does
    /// nothing.
    pub fn finish(&self) {
        self.inner.finish();
    }

    /// Wraps an iterator so every item it yields advances a new task by one step.
    pub fn track<I>(&self, items: I, description: impl Into<String>) -> Track<I::IntoIter>
    where
        I: IntoIterator,
        I::IntoIter: ExactSizeIterator,
    {
        let items = items.into_iter();
        let task = self.add_task(description, items.len() as u64);
        Track {
            items,
            task,
            owned: None,
            pending: false,
        }
    }
}

impl Default for Progress {
    fn default() -> Progress {
        Progress::new()
    }
}

impl fmt::Debug for Progress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Progress").finish_non_exhaustive()
    }
}

impl Renderable for Progress {
    fn render(&self, width: usize) -> Vec<Segment> {
        self.inner
            .core
            .render_once(width, self.inner.core.console.capabilities())
    }

    fn render_with(&self, width: usize, caps: &Capabilities) -> Vec<Segment> {
        self.inner.core.render_once(width, caps)
    }
}

/// Renders for the standard output profile without the final newline, so `println!("{progress}")`
/// prints the current frame once.
impl fmt::Display for Progress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let caps = capabilities(Stream::Stdout);
        let width = usize::from(caps.width);
        let ansi = to_ansi(&crop_lines(self.render_with(width, &caps), width), &caps);
        f.write_str(ansi.strip_suffix('\n').unwrap_or(&ansi))
    }
}

/// Builds a [`Progress`]; made by [`Progress::builder`].
///
/// ```
/// use hud::{BarColumn, MofNCompleteColumn, Progress, TaskProgressColumn, TextColumn};
///
/// let progress = Progress::builder()
///     .column(TextColumn::new("{task.description}"))
///     .column(BarColumn::new().bar_width(30))
///     .column(TaskProgressColumn::new())
///     .column(MofNCompleteColumn::new())
///     .disable(true)
///     .build();
/// # let _ = progress;
/// ```
pub struct ProgressBuilder {
    columns: Vec<ProgressColumn>,
    console: Console,
    transient: bool,
    disable: bool,
    auto_refresh: bool,
    refresh_per_second: f64,
    speed_period: f64,
    clock: Option<Clock>,
    writer: Option<Box<dyn Write + Send>>,
    interactive: Option<bool>,
}

impl ProgressBuilder {
    fn new() -> ProgressBuilder {
        ProgressBuilder {
            columns: Vec::new(),
            console: Console::stdout(),
            transient: false,
            disable: false,
            auto_refresh: true,
            refresh_per_second: 10.0,
            speed_period: SPEED_PERIOD,
            clock: None,
            writer: None,
            interactive: None,
        }
    }

    /// Adds a column after the ones already added. With none added, the default columns are
    /// used: description, bar, percentage and time remaining.
    #[must_use]
    pub fn column(mut self, column: impl Into<ProgressColumn>) -> ProgressBuilder {
        self.columns.push(column.into());
        self
    }

    /// The console the display draws on (default: standard output).
    #[must_use]
    pub fn console(mut self, console: Console) -> ProgressBuilder {
        self.console = console;
        self
    }

    /// Removes the display from the screen when it ends instead of leaving the last frame.
    #[must_use]
    pub fn transient(mut self, transient: bool) -> ProgressBuilder {
        self.transient = transient;
        self
    }

    /// Draws nothing at all. The display can still be rendered by hand with
    /// [`Console::render_to_string`].
    #[must_use]
    pub fn disable(mut self, disable: bool) -> ProgressBuilder {
        self.disable = disable;
        self
    }

    /// Whether a background thread redraws the display about ten times a second on a terminal
    /// (default: yes). Without it the display draws when a task is added, on
    /// [`Progress::refresh`] and when it ends.
    #[must_use]
    pub fn auto_refresh(mut self, auto_refresh: bool) -> ProgressBuilder {
        self.auto_refresh = auto_refresh;
        self
    }

    /// How often the background thread redraws (default 10).
    #[must_use]
    pub fn refresh_per_second(mut self, per_second: f64) -> ProgressBuilder {
        self.refresh_per_second = per_second;
        self
    }

    /// How many seconds of history the time remaining estimate uses (default 30).
    #[must_use]
    pub fn speed_estimate_period(mut self, seconds: f64) -> ProgressBuilder {
        self.speed_period = seconds;
        self
    }

    /// Replaces the clock, which returns seconds, so times are the same on every run.
    #[must_use]
    pub fn clock(mut self, clock: impl Fn() -> f64 + Send + Sync + 'static) -> ProgressBuilder {
        self.clock = Some(Arc::new(clock));
        self
    }

    /// Writes to `writer` instead of the console's stream.
    #[must_use]
    pub fn writer(mut self, writer: impl Write + Send + 'static) -> ProgressBuilder {
        self.writer = Some(Box::new(writer));
        self
    }

    /// Forces redrawing in place (`true`) or printing once at the end (`false`), in place of
    /// what the console reports.
    #[must_use]
    pub fn interactive(mut self, interactive: bool) -> ProgressBuilder {
        self.interactive = Some(interactive);
        self
    }

    /// Finishes the display.
    pub fn build(self) -> Progress {
        let columns = if self.columns.is_empty() {
            vec![
                ProgressColumn::Text(TextColumn::new("{task.description}")),
                ProgressColumn::Bar(BarColumn::new()),
                ProgressColumn::TaskProgress(TaskProgressColumn::new()),
                ProgressColumn::TimeRemaining(TimeRemainingColumn::new()),
            ]
        } else {
            self.columns
        };
        let track_speed = columns
            .iter()
            .any(|column| matches!(column, ProgressColumn::TimeRemaining(_)));
        let clock = self.clock.unwrap_or_else(|| {
            let origin = Instant::now();
            Arc::new(move || origin.elapsed().as_secs_f64())
        });
        let interactive = self
            .interactive
            .unwrap_or(self.console.capabilities().interactive);
        let auto_refresh =
            (self.auto_refresh && interactive && !self.disable && self.refresh_per_second > 0.0)
                .then(|| Duration::from_secs_f64(1.0 / self.refresh_per_second));
        let origins = vec![None; columns.len()];
        let core = Arc::new(Core {
            shared: Arc::new(Shared {
                clock,
                track_speed,
                period: self.speed_period,
                layout_version: AtomicU64::new(0),
            }),
            columns,
            tasks: RwLock::new(Vec::new()),
            console: self.console,
            interactive,
            transient: self.transient,
            disable: self.disable,
            out: Mutex::new(Out {
                writer: self.writer,
                started: false,
                stopped: false,
                height: 0,
                origins,
                plan_key: None,
                plan: None,
                pairs: Vec::new(),
                buffer: String::new(),
            }),
        });
        Progress {
            inner: Arc::new(Inner {
                core,
                refresher: Mutex::new(None),
                auto_refresh,
            }),
        }
    }
}

impl fmt::Debug for ProgressBuilder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ProgressBuilder")
            .field("columns", &self.columns.len())
            .finish_non_exhaustive()
    }
}

/// An iterator that advances a progress task once per item; made by [`Progress::track`] and
/// [`track`].
pub struct Track<I> {
    items: I,
    task: Task,
    owned: Option<Progress>,
    pending: bool,
}

impl<I: Iterator> Iterator for Track<I> {
    type Item = I::Item;

    fn next(&mut self) -> Option<I::Item> {
        if self.pending {
            self.task.advance(1);
            self.pending = false;
        }
        let item = self.items.next();
        if item.is_some() {
            self.pending = true;
        } else if let Some(progress) = &self.owned {
            progress.finish();
        }
        item
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.items.size_hint()
    }
}

impl<I: ExactSizeIterator> ExactSizeIterator for Track<I> {}

impl<I> fmt::Debug for Track<I> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Track").field("task", &self.task).finish()
    }
}

/// Iterates over `items` with a progress bar on standard output: one step per item, the display
/// ends when the loop does.
///
/// ```
/// let mut sum = 0;
/// for n in hud::track(0..3, "adding") {
///     sum += n;
/// }
/// assert_eq!(sum, 3);
/// ```
pub fn track<I>(items: I, description: impl Into<String>) -> Track<I::IntoIter>
where
    I: IntoIterator,
    I::IntoIter: ExactSizeIterator,
{
    let progress = Progress::new();
    let mut tracked = progress.track(items, description);
    tracked.owned = Some(progress);
    tracked
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::MofNCompleteColumn;

    fn console(color_system: ColorSystem) -> Console {
        Console::builder()
            .width(60)
            .color_system(color_system)
            .attributes(color_system != ColorSystem::None)
            .build()
    }

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

    fn live(color_system: ColorSystem, interactive: bool) -> (Progress, Capture) {
        let capture = Capture::default();
        let progress = Progress::builder()
            .console(console(color_system))
            .column(TextColumn::new("{task.description}"))
            .column(BarColumn::new().bar_width(10))
            .column(TaskProgressColumn::new())
            .column(MofNCompleteColumn::new())
            .writer(capture.clone())
            .interactive(interactive)
            .auto_refresh(false)
            .build();
        (progress, capture)
    }

    #[test]
    fn a_finished_task_marks_itself_once() {
        let progress = Progress::builder().disable(true).build();
        let task = progress.add_task("t", 3);
        task.advance(2);
        assert!(!task.is_finished());
        task.advance(1);
        assert!(task.is_finished());
        task.advance(5);
        assert_eq!(task.completed(), 8);
    }

    #[test]
    fn a_stream_that_is_not_interactive_prints_the_last_frame_once() {
        let (progress, capture) = live(ColorSystem::None, false);
        let task = progress.add_task("fetch", 4);
        task.advance(1);
        progress.refresh();
        assert_eq!(capture.text(), "");
        task.advance(3);
        progress.finish();
        assert_eq!(capture.text(), "fetch ━━━━━━━━━━ 100% 4/4\n");
        progress.finish();
        assert_eq!(capture.text().matches("fetch").count(), 1);
    }

    #[test]
    fn an_interactive_stream_redraws_in_place_and_restores_the_cursor() {
        let (progress, capture) = live(ColorSystem::None, true);
        let first = progress.add_task("a", 2);
        let _second = progress.add_task("b", 2);
        first.advance(1);
        progress.refresh();
        progress.finish();
        let text = capture.text();
        assert!(text.starts_with("\x1b[?25l"));
        assert!(text.contains("\r\x1b[2A\x1b[J"));
        assert!(text.ends_with("\n\x1b[?25h"));
        assert_eq!(text.matches("\x1b[?25h").count(), 1);
    }

    #[test]
    fn a_transient_display_leaves_nothing_behind() {
        let capture = Capture::default();
        let progress = Progress::builder()
            .console(console(ColorSystem::None))
            .writer(capture.clone())
            .interactive(true)
            .transient(true)
            .auto_refresh(false)
            .build();
        let task = progress.add_task("x", 1);
        task.advance(1);
        progress.finish();
        assert!(capture.text().ends_with("\r\x1b[1A\x1b[J\x1b[?25h"));
    }

    #[test]
    fn the_last_handle_ends_the_display() {
        let (progress, capture) = live(ColorSystem::None, false);
        let clone = progress.clone();
        let task = progress.add_task("only", 1);
        task.advance(1);
        drop(progress);
        assert_eq!(capture.text(), "");
        drop(clone);
        assert!(capture.text().contains("only"));
    }

    #[test]
    fn tracking_advances_once_per_item_and_finishes() {
        let (progress, capture) = live(ColorSystem::None, false);
        let seen: Vec<_> = progress.track(vec![1, 2, 3], "work").collect();
        assert_eq!(seen, [1, 2, 3]);
        progress.finish();
        assert!(capture.text().contains("100% 3/3"));
    }

    #[test]
    fn the_estimate_comes_from_the_speed_of_the_samples() {
        let now = Arc::new(Mutex::new(0.0_f64));
        let clock_now = Arc::clone(&now);
        let progress = Progress::builder()
            .console(console(ColorSystem::None))
            .column(TimeRemainingColumn::new())
            .clock(move || *lock(&clock_now))
            .disable(true)
            .build();
        let task = progress.add_task("t", 100);
        for step in 1..=5 {
            *lock(&now) = f64::from(step);
            task.advance(10);
        }
        let plain = console(ColorSystem::None).render_to_plain(&progress);
        // 4 samples after the first cover 40 steps in 4 s: 10 steps per second, 50 left.
        assert_eq!(plain, "0:00:05\n");
    }

    #[test]
    fn the_spinner_turns_with_the_clock() {
        let now = Arc::new(Mutex::new(0.0_f64));
        let clock_now = Arc::clone(&now);
        let progress = Progress::builder()
            .console(console(ColorSystem::None))
            .column(crate::model::SpinnerColumn::new())
            .clock(move || *lock(&clock_now))
            .disable(true)
            .build();
        progress.add_task("t", 10);
        let c = console(ColorSystem::None);
        assert_eq!(c.render_to_plain(&progress), "⠋\n");
        *lock(&now) = 0.085;
        assert_eq!(c.render_to_plain(&progress), "⠙\n");
    }
}
