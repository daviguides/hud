#![doc = include_str!("../../../README.md")]

mod console;
mod integrations;
mod model;
mod services;

pub use console::{Console, ConsoleBuilder, capabilities};
pub use hud_width as width;
pub use hud_width::{cell_width, cell_width_of_cluster, clusters, fold, pad, truncate};
pub use model::{
    Attribute, Capabilities, Color, ColorSystem, EnvSnapshot, Justify, MarkupError, Overflow,
    Renderable, Segment, Span, Stream, StreamInfo, Style, StyleError, Text,
};
pub use services::markup::escape;
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
