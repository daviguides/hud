use super::style::Style;

/// A run of text that has one style: the unit between a renderer and the bytes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Segment {
    /// The text of the run, with no escape codes.
    pub text: String,
    /// The style of the run.
    pub style: Style,
}

/// The least and the most width, in cells, a [`Renderable`] can be printed in.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Measure {
    /// The narrowest width that still shows the content: the longest word of a text.
    pub min: usize,
    /// The width that shows the content on as few lines as possible: the longest line.
    pub max: usize,
}

/// Something that can be printed by a [`Console`](crate::Console).
pub trait Renderable {
    /// Renders into styled runs for a line `width` cells wide. Line breaks are part of the
    /// runs.
    fn render(&self, width: usize) -> Vec<Segment>;

    /// The widths this renderable can be printed in, given that no more than `max_width` cells
    /// are available. A panel that fits its content asks for this. The default says any width
    /// from nothing up to `max_width`, which makes a fitting panel take all of it.
    fn measure(&self, max_width: usize) -> Measure {
        Measure {
            min: 0,
            max: max_width,
        }
    }
}
