use core::fmt;
use std::sync::Arc;

use super::capabilities::Capabilities;
use super::node::Node;
use super::padding::Padding;
use super::segment::{Measure, Renderable, Segment};
use super::style::Style;
use super::table::{BoxStyle, Table};
use super::text::Text;
use super::tree::Tree;

/// Where something sits in the room it has: a panel's title or subtitle on its border, an item
/// in the cell of a [`Columns`](crate::Columns).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Align {
    /// Next to the left corner.
    Left,
    /// In the middle.
    #[default]
    Center,
    /// Next to the right corner.
    Right,
}

/// The empty cells around a body: between a panel's border and its content, or around anything
/// wrapped in a [`Padding`](super::padding::Padding).
///
/// One number is all four sides, a pair is `(vertical, horizontal)` and four numbers are
/// `(top, right, bottom, left)`, as in CSS.
///
/// ```
/// use hud::Pad;
///
/// assert_eq!(Pad::from((1, 2)), Pad::from((1, 2, 1, 2)));
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Pad {
    /// Blank lines above the body.
    pub top: usize,
    /// Blank cells to the right of the body.
    pub right: usize,
    /// Blank lines below the body.
    pub bottom: usize,
    /// Blank cells to the left of the body.
    pub left: usize,
}

impl From<usize> for Pad {
    fn from(all: usize) -> Pad {
        Pad {
            top: all,
            right: all,
            bottom: all,
            left: all,
        }
    }
}

impl From<(usize, usize)> for Pad {
    fn from((vertical, horizontal): (usize, usize)) -> Pad {
        Pad {
            top: vertical,
            right: horizontal,
            bottom: vertical,
            left: horizontal,
        }
    }
}

impl From<(usize, usize, usize, usize)> for Pad {
    fn from((top, right, bottom, left): (usize, usize, usize, usize)) -> Pad {
        Pad {
            top,
            right,
            bottom,
            left,
        }
    }
}

impl Pad {
    pub(crate) fn is_none(self) -> bool {
        self.top == 0 && self.right == 0 && self.bottom == 0 && self.left == 0
    }
}

/// What a [`Panel`] holds: text, a table, a tree, another panel or any [`Renderable`].
///
/// A string is read as markup. Anything else goes through [`Body::new`].
#[derive(Clone)]
pub struct Body(
    pub(crate) Arc<dyn Renderable + Send + Sync>,
    pub(crate) Option<Arc<Text>>,
);

impl Body {
    /// Wraps any renderable as a panel body.
    pub fn new<R: Renderable + Send + Sync + 'static>(renderable: R) -> Body {
        Body(Arc::new(renderable), None)
    }

    /// Wraps text, remembering it as text: the cells of a grid lay text out as the cell asks
    /// (left-justified, ellipsis at the edge), which a plain renderable cannot be told.
    pub(crate) fn from_text(text: Text) -> Body {
        let text = Arc::new(text);
        Body(
            Arc::clone(&text) as Arc<dyn Renderable + Send + Sync>,
            Some(text),
        )
    }
}

/// A body is itself a renderable: it renders, measures and describes itself as the value it
/// holds, so one made from text, a table or a layout can be printed on its own.
impl Renderable for Body {
    fn render(&self, width: usize) -> Vec<Segment> {
        self.0.render(width)
    }

    fn render_with(&self, width: usize, caps: &Capabilities) -> Vec<Segment> {
        self.0.render_with(width, caps)
    }

    fn render_region(&self, width: usize, height: usize, caps: &Capabilities) -> Vec<Segment> {
        self.0.render_region(width, height, caps)
    }

    fn measure(&self, max_width: usize) -> Measure {
        self.0.measure(max_width)
    }

    fn node(&self) -> Node {
        self.0.node()
    }
}

impl fmt::Debug for Body {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Body(..)")
    }
}

impl From<&str> for Body {
    fn from(markup: &str) -> Body {
        Body::from_text(Text::from_markup(markup).unwrap_or_else(|_| Text::new(markup)))
    }
}

impl From<String> for Body {
    fn from(markup: String) -> Body {
        Body::from(markup.as_str())
    }
}

impl From<Text> for Body {
    fn from(text: Text) -> Body {
        Body::from_text(text)
    }
}

impl From<Table> for Body {
    fn from(table: Table) -> Body {
        Body::new(table)
    }
}

impl From<Padding> for Body {
    fn from(padding: Padding) -> Body {
        Body::new(padding)
    }
}

impl From<Tree> for Body {
    fn from(tree: Tree) -> Body {
        Body::new(tree)
    }
}

impl From<Panel> for Body {
    fn from(panel: Panel) -> Body {
        Body::new(panel)
    }
}

/// A border drawn around a body, with an optional title and subtitle.
///
/// By default the panel stretches to the console width and has rounded corners and a cell of
/// padding on each side. [`Panel::fit`] sizes it to its body instead. Title and subtitle are
/// markup; one longer than the border is cut.
///
/// ```
/// use hud::{Console, Panel};
///
/// let panel = Panel::new("Build [bold]ok[/]").title("Status");
/// let plain = Console::builder().width(24).plain().build().render_to_plain(&panel);
/// assert_eq!(
///     plain,
///     "╭─────── Status ───────╮\n│ Build ok             │\n╰──────────────────────╯\n"
/// );
/// ```
#[derive(Clone, Debug)]
pub struct Panel {
    pub(crate) body: Body,
    pub(crate) title: Option<String>,
    pub(crate) subtitle: Option<String>,
    pub(crate) title_align: Align,
    pub(crate) subtitle_align: Align,
    pub(crate) box_style: BoxStyle,
    pub(crate) border_style: Style,
    pub(crate) expand: bool,
    pub(crate) padding: Pad,
}

impl Panel {
    /// A panel as wide as the console, with rounded corners and one cell of side padding.
    pub fn new(body: impl Into<Body>) -> Panel {
        Panel {
            body: body.into(),
            title: None,
            subtitle: None,
            title_align: Align::Center,
            subtitle_align: Align::Center,
            box_style: BoxStyle::Rounded,
            border_style: Style::new(),
            expand: true,
            padding: Pad::from((0, 1)),
        }
    }

    /// A panel just wide enough for its body.
    pub fn fit(body: impl Into<Body>) -> Panel {
        Panel::new(body).expand(false)
    }

    /// The text in the top border, read as markup.
    #[must_use]
    pub fn title(mut self, title: impl Into<String>) -> Panel {
        self.title = Some(title.into());
        self
    }

    /// The text in the bottom border, read as markup.
    #[must_use]
    pub fn subtitle(mut self, subtitle: impl Into<String>) -> Panel {
        self.subtitle = Some(subtitle.into());
        self
    }

    /// Where the title sits (default: centered).
    #[must_use]
    pub fn title_align(mut self, align: Align) -> Panel {
        self.title_align = align;
        self
    }

    /// Where the subtitle sits (default: centered).
    #[must_use]
    pub fn subtitle_align(mut self, align: Align) -> Panel {
        self.subtitle_align = align;
        self
    }

    /// The border characters (default: rounded).
    #[must_use]
    pub fn box_style(mut self, box_style: BoxStyle) -> Panel {
        self.box_style = box_style;
        self
    }

    /// The style of the border and of the title and subtitle under their own markup.
    #[must_use]
    pub fn border_style(mut self, style: Style) -> Panel {
        self.border_style = style;
        self
    }

    /// Whether the panel stretches to the console width (the default) or fits its body.
    #[must_use]
    pub fn expand(mut self, expand: bool) -> Panel {
        self.expand = expand;
        self
    }

    /// The empty space around the body: one number for all sides, `(vertical, horizontal)` or
    /// `(top, right, bottom, left)` (default: no vertical padding and one cell on each side).
    #[must_use]
    pub fn padding(mut self, padding: impl Into<Pad>) -> Panel {
        self.padding = padding.into();
        self
    }
}
