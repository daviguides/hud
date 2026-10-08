//! What each widget says, as a [`Node`]: the structure behind the JSON format. Pure functions
//! over the model; nothing here knows a width, a style or a terminal.

use crate::model::{
    Body, Columns, Direction, ErrorReport, Justify, Layout, Node, NodeKind, Padding, Panel,
    Splitter, Table, Text, Tree,
};

/// Markup as the plain text it prints: tags removed. Markup that does not parse stays as
/// written, the way printing treats it.
pub(crate) fn plain_of(markup: &str) -> String {
    Text::from_markup(markup).map_or_else(|_| markup.to_string(), |text| text.plain().to_string())
}

fn optional(markup: &Option<String>) -> Option<String> {
    markup.as_deref().map(plain_of)
}

fn body_node(body: &Body) -> Node {
    body.0.node()
}

pub(crate) fn markup_node(markup: &str) -> Node {
    Node(NodeKind::Text(plain_of(markup)))
}

pub(crate) fn text_node(text: &Text) -> Node {
    Node(NodeKind::Text(text.plain().to_string()))
}

pub(crate) fn table_node(table: &Table) -> Node {
    Node(NodeKind::Table {
        title: optional(&table.title),
        caption: optional(&table.caption),
        columns: table
            .columns
            .iter()
            .map(|column| {
                let justify = match column.justify {
                    Justify::Default => Justify::Left,
                    other => other,
                };
                (plain_of(&column.header), justify)
            })
            .collect(),
        rows: table
            .rows
            .iter()
            .map(|row| row.iter().map(|cell| plain_of(cell)).collect())
            .collect(),
    })
}

pub(crate) fn panel_node(panel: &Panel) -> Node {
    Node(NodeKind::Panel {
        title: optional(&panel.title),
        subtitle: optional(&panel.subtitle),
        body: Box::new(body_node(&panel.body)),
    })
}

pub(crate) fn tree_node(tree: &Tree) -> Node {
    Node(NodeKind::Tree {
        label: plain_of(&tree.label),
        children: tree.children.iter().map(tree_node).collect(),
    })
}

pub(crate) fn error_node(report: &ErrorReport) -> Node {
    Node(NodeKind::Error {
        message: report.message.clone(),
        causes: report.causes.clone(),
        hint: report.hint.clone(),
    })
}

pub(crate) fn columns_node(columns: &Columns) -> Node {
    Node(NodeKind::Columns {
        title: optional(&columns.title),
        items: columns.items.iter().map(body_node).collect(),
    })
}

pub(crate) fn layout_node(layout: &Layout) -> Node {
    let direction = if layout.children.is_empty() {
        Direction::None
    } else {
        match layout.splitter {
            Splitter::Row => Direction::Row,
            Splitter::Column => Direction::Column,
        }
    };
    Node(NodeKind::Layout {
        name: layout.name.clone(),
        ratio: layout.ratio as u64,
        size: layout.size.map(|size| size as u64),
        visible: layout.visible,
        direction,
        content: layout.body.as_ref().map(|body| Box::new(body_node(body))),
        children: layout.children.iter().map(layout_node).collect(),
    })
}

pub(crate) fn padding_node(padding: &Padding) -> Node {
    Node(NodeKind::Padding {
        top: padding.pad.top as u64,
        right: padding.pad.right as u64,
        bottom: padding.pad.bottom as u64,
        left: padding.pad.left as u64,
        content: Box::new(body_node(&padding.body)),
    })
}
