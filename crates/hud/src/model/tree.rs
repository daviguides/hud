use super::style::Style;

/// A label with nested children, printed with guide lines.
///
/// Labels are markup. The guide lines are thin by default, heavy when the guide style is bold
/// and double when it is `underline2`.
///
/// ```
/// use hud::{Console, Tree};
///
/// let tree = Tree::new("hud/")
///     .child(Tree::new("src/").child("lib.rs").child("console.rs"))
///     .child("README.md");
/// let plain = Console::builder().width(40).plain().build().render_to_plain(&tree);
/// assert_eq!(
///     plain,
///     "hud/\n├── src/\n│   ├── lib.rs\n│   └── console.rs\n└── README.md\n"
/// );
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Tree {
    pub(crate) label: String,
    pub(crate) guide_style: Style,
    pub(crate) children: Vec<Tree>,
}

impl Tree {
    /// A node with no children; `label` is read as markup.
    pub fn new(label: impl Into<String>) -> Tree {
        Tree {
            label: label.into(),
            guide_style: Style::new(),
            children: Vec::new(),
        }
    }

    /// The style of the guide lines below this node, added over the style of its ancestors.
    #[must_use]
    pub fn guide_style(mut self, style: Style) -> Tree {
        self.guide_style = style;
        self
    }

    /// Adds a child; a string is a node with that label.
    #[must_use]
    pub fn child(mut self, child: impl Into<Tree>) -> Tree {
        self.children.push(child.into());
        self
    }

    /// Adds a child to a tree built in a loop and returns it, so it can get children of its own.
    pub fn add(&mut self, label: impl Into<String>) -> &mut Tree {
        self.children.push(Tree::new(label));
        let last = self.children.len() - 1;
        &mut self.children[last]
    }
}

impl From<&str> for Tree {
    fn from(label: &str) -> Tree {
        Tree::new(label)
    }
}

impl From<String> for Tree {
    fn from(label: String) -> Tree {
        Tree::new(label)
    }
}
