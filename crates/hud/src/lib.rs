#![doc = include_str!("../../../README.md")]

mod console;
mod integrations;
mod live;
mod model;
mod progress;
mod refresh;
mod services;

pub use console::{Console, ConsoleBuilder, capabilities, report};
pub use hud_width as width;
pub use hud_width::{cell_width, cell_width_of_cluster, clusters, fold, pad, truncate};
pub use live::{Live, LiveBuilder};
pub use model::{
    Align, Attribute, BarColumn, Body, BoxStyle, Capabilities, Color, ColorSystem, Column, Columns,
    EnvSnapshot, ErrorReport, Format, Group, Justify, Layout, MarkupError, Measure,
    MofNCompleteColumn, Node, NodeError, Overflow, Pad, Padding, Panel, ProgressColumn, Renderable,
    Segment, Span, SpinnerColumn, Stream, StreamInfo, Style, StyleError, Table, TaskProgressColumn,
    Text, TextColumn, TimeElapsedColumn, TimeRemainingColumn, Tree, VerticalOverflow,
};
pub use progress::{Progress, ProgressBuilder, Task, TaskUpdate, Track, track};
pub use services::markup::escape;

/// The width of `text` in terminal cells: Rich's `cell_len`, the same as [`cell_width`].
///
/// ```
/// assert_eq!(hud::cell_len("日本語"), 6);
/// ```
pub fn cell_len(text: &str) -> usize {
    cell_width(text)
}
pub use services::resolve::resolve;

#[doc(hidden)]
pub use console::print_markup as __print_markup;

/// Prints formatted text to standard output as markup, with a newline: `[bold red]error[/]`
/// styles the text, `\[` prints a bracket, and what the terminal cannot show is left out.
/// Escape values you do not control with [`escape`].
///
/// ```
/// hud::println!("[bold]{}[/] done in [green]{}s[/]", "build", 3);
/// ```
#[macro_export]
macro_rules! println {
    () => {
        $crate::__print_markup($crate::Stream::Stdout, "")
    };
    ($($arg:tt)*) => {
        $crate::__print_markup($crate::Stream::Stdout, &::std::format!($($arg)*))
    };
}

/// Like [`println!`] on standard error.
///
/// ```
/// hud::eprintln!("[yellow]warning[/]: {} files skipped", 3);
/// ```
#[macro_export]
macro_rules! eprintln {
    () => {
        $crate::__print_markup($crate::Stream::Stderr, "")
    };
    ($($arg:tt)*) => {
        $crate::__print_markup($crate::Stream::Stderr, &::std::format!($($arg)*))
    };
}
