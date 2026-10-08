//! Pure data: terminal capabilities and the inputs that decide them, colors, styles and text.

mod capabilities;
mod color;
mod columns;
mod error;
mod group;
mod layout;
mod live;
mod padding;
pub(crate) mod palette;
mod panel;
mod platform;
mod progress;
mod report;
mod segment;
mod style;
mod table;
mod text;
mod tree;

pub use capabilities::{Capabilities, ColorSystem, EnvSnapshot, Stream, StreamInfo};
pub use color::Color;
pub use columns::Columns;
pub use error::{MarkupError, StyleError};
pub use group::Group;
pub use layout::Layout;
pub(crate) use layout::Splitter;
pub use live::VerticalOverflow;
pub use padding::Padding;
pub use panel::{Align, Body, Pad, Panel};
pub(crate) use platform::WindowsFacts;
pub(crate) use progress::TaskSnapshot;
pub use progress::{
    BarColumn, MofNCompleteColumn, ProgressColumn, SpinnerColumn, TaskProgressColumn, TextColumn,
    TimeElapsedColumn, TimeRemainingColumn,
};
pub use report::ErrorReport;
pub use segment::{Measure, Renderable, Segment};
pub use style::{Attribute, Style};
pub use table::{BoxStyle, Column, Table};
pub(crate) use text::clean;
pub use text::{Justify, Overflow, Span, Text};
pub use tree::Tree;
