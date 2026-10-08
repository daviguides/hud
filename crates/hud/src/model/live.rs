/// What a live display does with a frame taller than the terminal.
///
/// ```
/// use hud::{Live, VerticalOverflow};
///
/// let live = Live::builder().vertical_overflow(VerticalOverflow::Crop).disable(true).build();
/// # let _ = live;
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum VerticalOverflow {
    /// Keeps the lines that fit, less one, and ends with a line of `...`.
    #[default]
    Ellipsis,
    /// Keeps the lines that fit and drops the rest.
    Crop,
    /// Shows every line, so the display scrolls the terminal.
    Visible,
}
