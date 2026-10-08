use core::ops::{Index, IndexMut};

use super::columns::Columns;
use super::group::Group;
use super::panel::{Body, Panel};
use super::table::Table;
use super::text::Text;
use super::tree::Tree;

/// How a layout divides its space among its children.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum Splitter {
    /// Children side by side.
    Row,
    /// Children stacked.
    #[default]
    Column,
}

/// A region of the terminal divided into rows and columns, each leaf showing any renderable.
///
/// A layout is as tall as the terminal: with a 24-row console it prints 24 lines. A child has a
/// fixed [`size`](Layout::size) or shares what is left by [`ratio`](Layout::ratio), never less
/// than its [`minimum_size`](Layout::minimum_size). A layout is split with [`Layout::row`] and
/// [`Layout::column`] (children side by side or stacked), or later with
/// [`split_row`](Layout::split_row) and [`split_column`](Layout::split_column). A named child is
/// reached by `layout["name"]` or [`Layout::get`], and its content replaced with
/// [`update`](Layout::update).
///
/// A leaf with no renderable ([`Layout::empty`]) is blank. Rich draws a placeholder panel there.
///
/// ```
/// use hud::{Console, Layout};
///
/// let layout = Layout::column([
///     Layout::new("[bold]header[/]").name("header").size(1),
///     Layout::row([Layout::new("left"), Layout::new("right")]).name("body"),
/// ]);
/// let console = Console::builder().width(12).height(3).plain().build();
/// assert_eq!(console.render_to_plain(&layout), "header      \nleft  right \n            \n");
/// ```
#[derive(Clone, Debug)]
pub struct Layout {
    pub(crate) name: Option<String>,
    pub(crate) size: Option<usize>,
    pub(crate) minimum_size: usize,
    pub(crate) ratio: usize,
    pub(crate) visible: bool,
    pub(crate) splitter: Splitter,
    pub(crate) body: Option<Body>,
    pub(crate) children: Vec<Layout>,
}

impl Layout {
    fn blank(body: Option<Body>) -> Layout {
        Layout {
            name: None,
            size: None,
            minimum_size: 1,
            ratio: 1,
            visible: true,
            splitter: Splitter::Column,
            body,
            children: Vec::new(),
        }
    }

    /// A leaf showing `body`; a string is read as markup.
    pub fn new(body: impl Into<Body>) -> Layout {
        Layout::blank(Some(body.into()))
    }

    /// A leaf with nothing in it yet: blank until [`Layout::update`] gives it content.
    pub fn empty() -> Layout {
        Layout::blank(None)
    }

    /// `children` side by side.
    pub fn row(children: impl IntoIterator<Item = Layout>) -> Layout {
        let mut layout = Layout::empty();
        layout.split_row(children);
        layout
    }

    /// `children` stacked, the first on top.
    pub fn column(children: impl IntoIterator<Item = Layout>) -> Layout {
        let mut layout = Layout::empty();
        layout.split_column(children);
        layout
    }

    /// A name to find this layout by.
    #[must_use]
    pub fn name(mut self, name: impl Into<String>) -> Layout {
        self.name = Some(name.into());
        self
    }

    /// A fixed size, in columns for the child of a row and in lines for the child of a column.
    /// Without one the child is flexible. `0` means flexible.
    #[must_use]
    pub fn size(mut self, size: usize) -> Layout {
        self.size = Some(size);
        self
    }

    /// The least a flexible child can be given (default 1).
    #[must_use]
    pub fn minimum_size(mut self, minimum_size: usize) -> Layout {
        self.minimum_size = minimum_size;
        self
    }

    /// The share of the flexible space this child takes, against its siblings' (default 1).
    #[must_use]
    pub fn ratio(mut self, ratio: usize) -> Layout {
        self.ratio = ratio;
        self
    }

    /// Whether the child takes part in the layout (default `true`). An invisible child takes no
    /// space and prints nothing.
    #[must_use]
    pub fn visible(mut self, visible: bool) -> Layout {
        self.visible = visible;
        self
    }

    /// Replaces the children with `children` side by side.
    pub fn split_row(&mut self, children: impl IntoIterator<Item = Layout>) {
        self.splitter = Splitter::Row;
        self.children = children.into_iter().collect();
    }

    /// Replaces the children with `children` stacked.
    pub fn split_column(&mut self, children: impl IntoIterator<Item = Layout>) {
        self.splitter = Splitter::Column;
        self.children = children.into_iter().collect();
    }

    /// Removes the children: the layout is a leaf again.
    pub fn unsplit(&mut self) {
        self.children.clear();
    }

    /// Replaces what a leaf shows.
    pub fn update(&mut self, body: impl Into<Body>) {
        self.body = Some(body.into());
    }

    /// The layout called `name`, this one or a descendant.
    pub fn get(&self, name: &str) -> Option<&Layout> {
        if self.name.as_deref() == Some(name) {
            return Some(self);
        }
        self.children.iter().find_map(|child| child.get(name))
    }

    /// Like [`Layout::get`], to change the layout found.
    pub fn get_mut(&mut self, name: &str) -> Option<&mut Layout> {
        if self.name.as_deref() == Some(name) {
            return Some(self);
        }
        self.children
            .iter_mut()
            .find_map(|child| child.get_mut(name))
    }
}

/// The layout called `name`, as Rich's `layout["name"]`.
///
/// # Panics
///
/// When there is no layout of that name, as indexing a slice out of range does. Use
/// [`Layout::get`] to handle a missing name.
#[allow(clippy::panic)]
impl Index<&str> for Layout {
    type Output = Layout;

    fn index(&self, name: &str) -> &Layout {
        match self.get(name) {
            Some(layout) => layout,
            None => panic!("no layout called {name:?}"),
        }
    }
}

/// See [`Index`] for the layout above; panics the same way.
#[allow(clippy::panic)]
impl IndexMut<&str> for Layout {
    fn index_mut(&mut self, name: &str) -> &mut Layout {
        match self.get_mut(name) {
            Some(layout) => layout,
            None => panic!("no layout called {name:?}"),
        }
    }
}

impl From<Layout> for Body {
    fn from(layout: Layout) -> Body {
        Body::new(layout)
    }
}

macro_rules! layout_from {
    ($($ty:ty),* $(,)?) => {
        $(impl From<$ty> for Layout {
            fn from(content: $ty) -> Layout {
                Layout::new(content)
            }
        })*
    };
}

layout_from!(&str, String, Text, Table, Tree, Panel, Group, Columns);
