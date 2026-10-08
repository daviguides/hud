use super::segment::Segment;
use super::text::Justify;

/// What a renderable says, with nothing of how it looks: the structure behind
/// [`Format::Json`](crate::Format::Json).
///
/// A node is plain data. Text fields hold plain strings (markup already read), and there is no
/// style, color, width, wrapping or timing in it, so one value gives one node on every console.
/// [`Renderable::node`](crate::Renderable::node) makes one; [`Node::to_json`] writes it as the
/// document `hud/1` and [`Node::from_json`] reads such a document back.
///
/// ```
/// use hud::Node;
///
/// let node = Node::group([Node::text("build"), Node::text("3 crates")]);
/// let json = node.to_json();
/// assert_eq!(Node::from_json(&json).unwrap(), node);
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Node(pub(crate) NodeKind);

/// A node, by what it holds. The shape of every variant is the table of `structured-json.md`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum NodeKind {
    Text(String),
    Table {
        title: Option<String>,
        caption: Option<String>,
        columns: Vec<(String, Justify)>,
        rows: Vec<Vec<String>>,
    },
    Panel {
        title: Option<String>,
        subtitle: Option<String>,
        body: Box<Node>,
    },
    Tree {
        label: String,
        children: Vec<Node>,
    },
    Progress(Vec<TaskState>),
    Error {
        message: String,
        causes: Vec<String>,
        hint: Option<String>,
    },
    Columns {
        title: Option<String>,
        items: Vec<Node>,
    },
    Layout {
        name: Option<String>,
        ratio: u64,
        size: Option<u64>,
        visible: bool,
        direction: Direction,
        content: Option<Box<Node>>,
        children: Vec<Node>,
    },
    Group(Vec<Node>),
    Padding {
        top: u64,
        right: u64,
        bottom: u64,
        left: u64,
        content: Box<Node>,
    },
}

/// One task of a progress display at the moment it was described.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct TaskState {
    pub(crate) description: String,
    pub(crate) completed: u64,
    pub(crate) total: u64,
    pub(crate) finished: bool,
    pub(crate) visible: bool,
}

/// How a layout divides its children.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Direction {
    None,
    Row,
    Column,
}

impl Node {
    /// A node that holds text. The string is taken as it is, with no markup read.
    pub fn text(text: impl Into<String>) -> Node {
        Node(NodeKind::Text(text.into()))
    }

    /// A node that holds other nodes one after the other.
    pub fn group(items: impl IntoIterator<Item = Node>) -> Node {
        Node(NodeKind::Group(items.into_iter().collect()))
    }
}

/// The text of rendered runs with no style: lines without trailing spaces and without the final
/// line break. What a custom renderable that does not describe itself becomes.
pub(crate) fn plain_lines(segments: &[Segment]) -> String {
    let mut text = String::new();
    for segment in segments {
        text.push_str(&segment.text);
    }
    let lines: Vec<&str> = text.split('\n').map(str::trim_end).collect();
    lines.join("\n").trim_end_matches('\n').to_string()
}
