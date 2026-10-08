//! Pure data: terminal capabilities and the inputs that decide them, colors, styles and text.

mod capabilities;
mod color;
mod error;
pub(crate) mod palette;
mod panel;
mod segment;
mod style;
mod table;
mod text;
mod tree;

pub use capabilities::{Capabilities, ColorSystem, EnvSnapshot, Stream, StreamInfo};
pub use color::Color;
pub use error::{MarkupError, StyleError};
pub use panel::{Align, Body, Padding, Panel};
pub use segment::{Measure, Renderable, Segment};
pub use style::{Attribute, Style};
pub use table::{BoxStyle, Column, Table};
pub(crate) use text::clean;
pub use text::{Justify, Overflow, Span, Text};
pub use tree::Tree;
