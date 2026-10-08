use super::capabilities::Capabilities;
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

    /// Like [`Renderable::render`] for output with these capabilities. Most widgets look the
    /// same whatever the terminal can show and keep this default; a progress bar leaves out its
    /// track when there is no color.
    fn render_with(&self, width: usize, _caps: &Capabilities) -> Vec<Segment> {
        self.render(width)
    }

    /// Like [`Renderable::render_with`] for a box `height` lines tall. Most widgets are as tall
    /// as their content and keep this default; a [`Layout`](crate::Layout) fills the height it is
    /// given, so one placed inside another layout takes the height of its region.
    fn render_region(&self, width: usize, height: usize, caps: &Capabilities) -> Vec<Segment> {
        let _ = height;
        self.render_with(width, caps)
    }

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
