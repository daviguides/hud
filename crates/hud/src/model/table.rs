use super::style::Style;
use super::text::{Justify, Overflow};

/// The line-drawing characters around and between the cells of a [`Table`].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum BoxStyle {
    /// `+`, `-` and `|`: safe on any terminal.
    Ascii,
    /// Single lines with rounded corners.
    Rounded,
    /// No outer lines: a rule under the header only.
    Simple,
    /// Heavy lines everywhere.
    Heavy,
    /// Double lines everywhere.
    Double,
    /// Thin separators with open corners.
    Minimal,
    /// Single lines with square corners.
    Square,
    /// Square corners, a heavy line around the header and thin lines elsewhere.
    #[default]
    HeavyHead,
}

/// One column of a [`Table`]: the header text and how its cells are laid out.
///
/// ```
/// use hud::{Column, Justify};
///
/// let column = Column::new("Downloads").justify(Justify::Right).no_wrap(true);
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Column {
    pub(crate) header: String,
    pub(crate) justify: Justify,
    pub(crate) style: Style,
    pub(crate) header_style: Style,
    pub(crate) no_wrap: bool,
    pub(crate) overflow: Overflow,
    pub(crate) width: Option<usize>,
    pub(crate) min_width: Option<usize>,
    pub(crate) max_width: Option<usize>,
}

impl Column {
    /// A left aligned column whose header is `header`, read as markup.
    pub fn new(header: impl Into<String>) -> Column {
        Column {
            header: header.into(),
            justify: Justify::Left,
            style: Style::new(),
            header_style: Style::new(),
            no_wrap: false,
            overflow: Overflow::Ellipsis,
            width: None,
            min_width: None,
            max_width: None,
        }
    }

    /// Places the cells of the column inside its width.
    #[must_use]
    pub fn justify(mut self, justify: Justify) -> Column {
        self.justify = justify;
        self
    }

    /// The style every cell of the column starts from; markup in a cell is applied over it.
    #[must_use]
    pub fn style(mut self, style: Style) -> Column {
        self.style = style;
        self
    }

    /// A style added to this column's header over the table's header style.
    #[must_use]
    pub fn header_style(mut self, style: Style) -> Column {
        self.header_style = style;
        self
    }

    /// Turns word wrapping off: a cell that is too wide is cut by the overflow setting.
    #[must_use]
    pub fn no_wrap(mut self, no_wrap: bool) -> Column {
        self.no_wrap = no_wrap;
        self
    }

    /// What happens to text that does not fit (default: an ellipsis).
    #[must_use]
    pub fn overflow(mut self, overflow: Overflow) -> Column {
        self.overflow = overflow;
        self
    }

    /// A fixed width for the cell text, without the one cell of padding on each side.
    #[must_use]
    pub fn width(mut self, width: usize) -> Column {
        self.width = Some(width);
        self
    }

    /// The narrowest cell text the column shrinks to, without padding.
    #[must_use]
    pub fn min_width(mut self, width: usize) -> Column {
        self.min_width = Some(width);
        self
    }

    /// The widest cell text the column grows to, without padding.
    #[must_use]
    pub fn max_width(mut self, width: usize) -> Column {
        self.max_width = Some(width);
        self
    }
}

impl From<&str> for Column {
    fn from(header: &str) -> Column {
        Column::new(header)
    }
}

impl From<String> for Column {
    fn from(header: String) -> Column {
        Column::new(header)
    }
}

/// A grid of text with a header row, printed with borders and fitted to the console width.
///
/// Headers, cells, title and caption are markup. Columns shrink to fit: the widest wrappable
/// column gives up cells first, and a border is never lost.
///
/// ```
/// use hud::{Column, Console, Justify, Table};
///
/// let table = Table::new()
///     .title("Build report")
///     .column("Crate")
///     .column(Column::new("Downloads").justify(Justify::Right))
///     .row(["clap", "12,400,000"])
///     .row(["syn", "[red]87,000,000[/]"]);
/// let plain = Console::builder().width(40).plain().build().render_to_plain(&table);
/// assert!(plain.contains("Build report"));
/// assert!(plain.contains("┃ Crate "));
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Table {
    pub(crate) title: Option<String>,
    pub(crate) caption: Option<String>,
    pub(crate) box_style: BoxStyle,
    pub(crate) show_lines: bool,
    pub(crate) header_style: Style,
    pub(crate) columns: Vec<Column>,
    pub(crate) rows: Vec<Vec<String>>,
}

impl Default for Table {
    fn default() -> Table {
        Table {
            title: None,
            caption: None,
            box_style: BoxStyle::default(),
            show_lines: false,
            header_style: Style::new().bold(),
            columns: Vec::new(),
            rows: Vec::new(),
        }
    }
}

impl Table {
    /// An empty table with heavy-head borders and bold headers.
    pub fn new() -> Table {
        Table::default()
    }

    /// The style of every header, in place of the default bold; a column's own header style is
    /// added over it.
    #[must_use]
    pub fn header_style(mut self, style: Style) -> Table {
        self.header_style = style;
        self
    }

    /// The text centered above the table, read as markup.
    #[must_use]
    pub fn title(mut self, title: impl Into<String>) -> Table {
        self.title = Some(title.into());
        self
    }

    /// The text centered below the table, read as markup.
    #[must_use]
    pub fn caption(mut self, caption: impl Into<String>) -> Table {
        self.caption = Some(caption.into());
        self
    }

    /// The border characters.
    #[must_use]
    pub fn box_style(mut self, box_style: BoxStyle) -> Table {
        self.box_style = box_style;
        self
    }

    /// Draws a rule between every two rows.
    #[must_use]
    pub fn show_lines(mut self, show_lines: bool) -> Table {
        self.show_lines = show_lines;
        self
    }

    /// Adds a column; a string is a left aligned column with that header.
    #[must_use]
    pub fn column(mut self, column: impl Into<Column>) -> Table {
        self.add_column(column);
        self
    }

    /// Adds a row of cells, each read as markup. A short row is completed with empty cells; a
    /// longer one adds columns with an empty header.
    #[must_use]
    pub fn row<I, S>(mut self, cells: I) -> Table
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.add_row(cells);
        self
    }

    /// Like [`Table::column`], for tables built in a loop.
    pub fn add_column(&mut self, column: impl Into<Column>) {
        self.columns.push(column.into());
    }

    /// Like [`Table::row`], for tables built in a loop.
    pub fn add_row<I, S>(&mut self, cells: I)
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let cells: Vec<String> = cells.into_iter().map(Into::into).collect();
        while self.columns.len() < cells.len() {
            self.columns.push(Column::new(""));
        }
        self.rows.push(cells);
    }
}
