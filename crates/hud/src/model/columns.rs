use super::panel::{Align, Body, Padding};

/// Items laid out in neat columns: as many columns as fit the width, filled left to right (or
/// top to bottom), each column as wide as its widest item.
///
/// An item is anything a [`Panel`](crate::Panel) accepts as a body: a string read as markup,
/// a [`Table`](crate::Table), a [`Tree`](crate::Tree), another panel. The defaults match Rich's
/// `Columns`: one cell of padding between columns and nothing at the edges.
///
/// ```
/// use hud::{Columns, Console};
///
/// let columns = Columns::new(["clap", "serde", "tokio", "hyper", "rayon", "regex"]);
/// let console = Console::builder().width(24).plain().build();
/// assert_eq!(
///     console.render_to_plain(&columns),
///     "clap  serde tokio hyper\nrayon regex            \n",
/// );
/// ```
#[derive(Clone, Debug)]
pub struct Columns {
    pub(crate) items: Vec<Body>,
    pub(crate) padding: Padding,
    pub(crate) width: Option<usize>,
    pub(crate) expand: bool,
    pub(crate) equal: bool,
    pub(crate) column_first: bool,
    pub(crate) right_to_left: bool,
    pub(crate) align: Option<Align>,
    pub(crate) title: Option<String>,
}

impl Default for Columns {
    fn default() -> Columns {
        Columns {
            items: Vec::new(),
            padding: Padding::from((0, 1)),
            width: None,
            expand: false,
            equal: false,
            column_first: false,
            right_to_left: false,
            align: None,
            title: None,
        }
    }
}

impl Columns {
    /// Columns of `items`; a string is read as markup.
    pub fn new<I>(items: I) -> Columns
    where
        I: IntoIterator,
        I::Item: Into<Body>,
    {
        Columns {
            items: items.into_iter().map(Into::into).collect(),
            ..Columns::default()
        }
    }

    /// Adds an item after the ones already in the columns.
    #[must_use]
    pub fn push(mut self, item: impl Into<Body>) -> Columns {
        self.items.push(item.into());
        self
    }

    /// The empty cells around each item (default one cell between columns): one number is all
    /// four sides, a pair is `(vertical, horizontal)`, four numbers are `(top, right, bottom,
    /// left)`. The padding at the edges of the grid is dropped and the padding between two
    /// columns is collapsed, as Rich does.
    #[must_use]
    pub fn padding(mut self, padding: impl Into<Padding>) -> Columns {
        self.padding = padding.into();
        self
    }

    /// A fixed width, in cells, for every column (default: measured from the items). As many
    /// columns as fit are used.
    #[must_use]
    pub fn width(mut self, width: usize) -> Columns {
        self.width = Some(width);
        self
    }

    /// Stretches the grid to the whole width of the console.
    #[must_use]
    pub fn expand(mut self, expand: bool) -> Columns {
        self.expand = expand;
        self
    }

    /// Makes every column as wide as the widest item.
    #[must_use]
    pub fn equal(mut self, equal: bool) -> Columns {
        self.equal = equal;
        self
    }

    /// Fills the first column from top to bottom, then the next, instead of row by row.
    #[must_use]
    pub fn column_first(mut self, column_first: bool) -> Columns {
        self.column_first = column_first;
        self
    }

    /// Starts each row from the right.
    #[must_use]
    pub fn right_to_left(mut self, right_to_left: bool) -> Columns {
        self.right_to_left = right_to_left;
        self
    }

    /// Places every item at the left, center or right of its cell (default: as the item
    /// comes).
    #[must_use]
    pub fn align(mut self, align: Align) -> Columns {
        self.align = Some(align);
        self
    }

    /// A title, read as markup, centered over the grid.
    #[must_use]
    pub fn title(mut self, title: impl Into<String>) -> Columns {
        self.title = Some(title.into());
        self
    }
}

impl From<Columns> for Body {
    fn from(columns: Columns) -> Body {
        Body::new(columns)
    }
}
