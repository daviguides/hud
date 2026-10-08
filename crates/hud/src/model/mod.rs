//! Pure data: terminal capabilities and the inputs that decide them, colors, styles and text.

mod capabilities;
mod color;
mod error;
pub(crate) mod palette;
mod segment;
mod style;
mod text;

pub use capabilities::{Capabilities, ColorSystem, EnvSnapshot, Stream, StreamInfo};
pub use color::Color;
pub use error::{MarkupError, StyleError};
pub use segment::{Renderable, Segment};
pub use style::{Attribute, Style};
pub use text::{Justify, Overflow, Span, Text};
