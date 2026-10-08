//! A tree spec to styled runs: one line per node, with the guide lines of its ancestors in front.

use super::frame::{markup_text, run, split_lines};
use super::layout::{Measurement, measure_renderable};
use super::render::render_text;
use crate::model::{Attribute, Measure, Segment, Style, Text, Tree};

/// Cells every guide takes.
const GUIDE_WIDTH: usize = 4;

/// The four guide shapes: blank, a line going on, a branch with more below it, the last branch.
#[derive(Clone, Copy)]
enum Guide {
    Space,
    Continue,
    Fork,
    End,
}

/// The guide shapes in thin, heavy and double lines.
const TREE_GUIDES: [[&str; 4]; 3] = [
    ["    ", "│   ", "├── ", "└── "],
    ["    ", "┃   ", "┣━━ ", "┗━━ "],
    ["    ", "║   ", "╠══ ", "╚══ "],
];

/// The connector a node sits on, or an ancestor's connector: whether that node is the last of
/// its siblings, and the style of the guide lines of its level.
struct Level {
    last: bool,
    style: Style,
}

fn guide_text(guide: Guide, style: &Style) -> &'static str {
    let set = if style.get(Attribute::Bold) == Some(true) {
        1
    } else if style.get(Attribute::Underline2) == Some(true) {
        2
    } else {
        0
    };
    TREE_GUIDES[set][guide as usize]
}

/// The segment of one guide. Heavy and double guides are chosen by the bold and `underline2`
/// of the style, and those two attributes are then turned off so only color and the others show.
fn guide_segment(guide: Guide, style: &Style) -> Segment {
    let off = Style::new()
        .attribute(Attribute::Bold, false)
        .attribute(Attribute::Underline2, false);
    run(guide_text(guide, style), &style.combine(&off))
}

/// The lines of one label, wrapped to `width` cells and cut to it, without padding.
fn label_lines(label: &str, width: usize) -> Vec<Vec<Segment>> {
    if width == 0 {
        return Vec::new();
    }
    split_lines(render_text(&markup_text(label), width), width, None)
}

struct Walk<'a> {
    width: usize,
    out: &'a mut Vec<Segment>,
}

impl Walk<'_> {
    /// Prints `node` and, below it, its children. `inherited` is the guide style in force where
    /// the node sits, `own` the connector the node hangs on (none for the root) and `ancestors`
    /// the connectors above it.
    fn node(&mut self, node: &Tree, inherited: &Style, own: Option<&Level>, ancestors: &[Level]) {
        let guide_style = inherited.combine(&node.guide_style);
        let depth = ancestors.len() + usize::from(own.is_some());
        let label_width = self.width.saturating_sub(depth * GUIDE_WIDTH);
        for (index, line) in label_lines(&node.label, label_width)
            .into_iter()
            .enumerate()
        {
            for level in ancestors {
                let guide = if level.last {
                    Guide::Space
                } else {
                    Guide::Continue
                };
                self.out.push(guide_segment(guide, &level.style));
            }
            if let Some(level) = own {
                let guide = match (index, level.last) {
                    (0, true) => Guide::End,
                    (0, false) => Guide::Fork,
                    (_, true) => Guide::Space,
                    (_, false) => Guide::Continue,
                };
                self.out.push(guide_segment(guide, &level.style));
            }
            self.out.extend(line);
            self.out.push(run("\n", &Style::new()));
        }
        let mut below: Vec<Level> = Vec::with_capacity(ancestors.len() + 1);
        for level in ancestors {
            below.push(Level {
                last: level.last,
                style: level.style.clone(),
            });
        }
        if let Some(level) = own {
            below.push(Level {
                last: level.last,
                style: level.style.clone(),
            });
        }
        let count = node.children.len();
        for (index, child) in node.children.iter().enumerate() {
            let level = Level {
                last: index + 1 == count,
                style: guide_style.clone(),
            };
            self.node(child, &guide_style, Some(&level), &below);
        }
    }
}

/// Renders `tree` for a console `width` cells wide.
pub(crate) fn render_tree(tree: &Tree, width: usize) -> Vec<Segment> {
    let mut out = Vec::new();
    Walk {
        width,
        out: &mut out,
    }
    .node(tree, &tree.guide_style, None, &[]);
    out
}

/// The widths a tree asks for when no more than `max_width` cells are available: the widest
/// label, each indented by its depth.
pub(crate) fn measure_tree(tree: &Tree, max_width: usize) -> Measure {
    fn visit(node: &Tree, depth: usize, room: i64, best: &mut Measurement) {
        let label: Text = markup_text(&node.label);
        let measured = measure_renderable(&label, room);
        let indent = (depth * GUIDE_WIDTH) as i64;
        best.min = best.min.max(measured.min + indent);
        best.max = best.max.max(measured.max + indent);
        for child in &node.children {
            visit(child, depth + 1, room, best);
        }
    }
    let mut best = Measurement { min: 0, max: 0 };
    visit(tree, 0, max_width as i64, &mut best);
    best.measure()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::render::to_plain;

    fn plain(tree: &Tree, width: usize) -> String {
        to_plain(&render_tree(tree, width))
    }

    #[test]
    fn guides_join_siblings_and_close_the_last() {
        let tree = Tree::new("root")
            .child(Tree::new("a").child("a1").child("a2"))
            .child("b");
        assert_eq!(
            plain(&tree, 40),
            "root\n├── a\n│   ├── a1\n│   └── a2\n└── b\n"
        );
    }

    #[test]
    fn a_bold_guide_style_draws_heavy_lines_without_the_bold() {
        let tree = Tree::new("r")
            .guide_style(Style::new().bold())
            .child("x")
            .child("y");
        assert_eq!(plain(&tree, 40), "r\n┣━━ x\n┗━━ y\n");
        let segments = render_tree(&tree, 40);
        assert!(
            segments
                .iter()
                .all(|s| s.style.get(Attribute::Bold) != Some(true))
        );
    }

    #[test]
    fn a_label_that_wraps_continues_the_guide_beside_its_lines() {
        let tree = Tree::new("r").child("aaaa bbbb").child("z");
        assert_eq!(plain(&tree, 11), "r\n├── aaaa \n│   bbbb\n└── z\n");
    }

    #[test]
    fn a_node_with_no_room_for_its_label_prints_no_line() {
        let tree = Tree::new("r").child("x");
        assert_eq!(plain(&tree, 2), "r\n");
        assert_eq!(plain(&tree, 4), "r\n");
        assert_eq!(plain(&tree, 5), "r\n└── x\n");
    }
}
