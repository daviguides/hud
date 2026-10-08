use super::panel::{Body, Pad};
use super::style::Style;

/// Blank space around any renderable, as Rich's `Padding(renderable, pad)`.
///
/// `pad` is one number for all four sides, `(vertical, horizontal)` or
/// `(top, right, bottom, left)`. The wrapped renderable fills the width that is left unless
/// [`Padding::expand`] says otherwise, and [`Padding::style`] paints the blank cells and the
/// renderable's own background.
///
/// ```
/// use hud::{Console, Padding};
///
/// let console = Console::builder().width(12).plain().build();
/// let padded = Padding::new("ok", (1, 2));
/// assert_eq!(console.render_to_plain(&padded), "            \n  ok        \n            \n");
/// ```
#[derive(Clone, Debug)]
pub struct Padding {
    pub(crate) body: Body,
    pub(crate) pad: Pad,
    pub(crate) style: Style,
    pub(crate) expand: bool,
}

impl Padding {
    /// Wraps `renderable` in `pad` blank cells. A string is read as markup.
    pub fn new(renderable: impl Into<Body>, pad: impl Into<Pad>) -> Padding {
        Padding {
            body: renderable.into(),
            pad: pad.into(),
            style: Style::new(),
            expand: true,
        }
    }

    /// The style of the blank cells; it is also the base style of the wrapped renderable.
    #[must_use]
    pub fn style(mut self, style: Style) -> Padding {
        self.style = style;
        self
    }

    /// Whether the padding takes the whole width (the default) or only what the renderable
    /// needs plus the padding.
    #[must_use]
    pub fn expand(mut self, expand: bool) -> Padding {
        self.expand = expand;
        self
    }
}
