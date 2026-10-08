use super::capabilities::Capabilities;
use super::node::{Node, NodeKind};
use super::panel::Body;
use super::segment::{Measure, Renderable, Segment};

/// Renderables printed one after the other, each on its own lines.
///
/// A string is read as markup, as in a [`Panel`](crate::Panel) body. A group is itself a
/// renderable, so it can be the body of a panel or an item of another group.
///
/// ```
/// use hud::{Console, Group, Panel};
///
/// let group = Group::new().push("[bold]build[/]").push("3 crates");
/// let plain = Console::builder().width(14).plain().build().render_to_plain(&Panel::new(group));
/// assert_eq!(plain, "╭────────────╮\n│ build      │\n│ 3 crates   │\n╰────────────╯\n");
/// ```
#[derive(Clone, Debug, Default)]
pub struct Group {
    pub(crate) items: Vec<Body>,
}

impl Group {
    /// An empty group.
    pub fn new() -> Group {
        Group::default()
    }

    /// Adds an item after the ones already in the group.
    #[must_use]
    pub fn push(mut self, item: impl Into<Body>) -> Group {
        self.items.push(item.into());
        self
    }
}

impl Renderable for Group {
    fn node(&self) -> Node {
        Node(NodeKind::Group(
            self.items.iter().map(|item| item.0.node()).collect(),
        ))
    }

    fn render(&self, width: usize) -> Vec<Segment> {
        self.items
            .iter()
            .flat_map(|item| item.0.render(width))
            .collect()
    }

    fn render_with(&self, width: usize, caps: &Capabilities) -> Vec<Segment> {
        self.items
            .iter()
            .flat_map(|item| item.0.render_with(width, caps))
            .collect()
    }

    fn measure(&self, max_width: usize) -> Measure {
        let mut measure = Measure { min: 0, max: 0 };
        for item in &self.items {
            let one = item.0.measure(max_width);
            measure.min = measure.min.max(one.min);
            measure.max = measure.max.max(one.max);
        }
        measure
    }
}

impl From<Group> for Body {
    fn from(group: Group) -> Body {
        Body::new(group)
    }
}
