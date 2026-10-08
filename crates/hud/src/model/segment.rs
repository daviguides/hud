use super::style::Style;

/// A run of text that has one style: the unit between a renderer and the bytes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Segment {
    /// The text of the run, with no escape codes.
    pub text: String,
    /// The style of the run.
    pub style: Style,
}

/// Something that can be printed by a [`Console`](crate::Console).
pub trait Renderable {
    /// Renders into styled runs for a line `width` cells wide. Line breaks are part of the
    /// runs.
    fn render(&self, width: usize) -> Vec<Segment>;
}
