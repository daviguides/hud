//! Glue: the declarative case schema rendered with comfy-table (table), owo-colors (style) and
//! hand-written code for what the three crates do not have (panel, tree, error layout, markup).
use crate::style::{Depth, St, paint, parse_style};
use crate::text::{
    Cell, natural_width, parse_markup, plain_cells, render_text, spans_to_ansi, vis_width, wrap,
};
use comfy_table::{
    CellAlignment, ColumnConstraint, ContentArrangement, ContentLineStyle, LineStyle, Table,
    TableStyle,
};
use serde_json::Value;

const BOXES: [(&str, [&str; 8]); 8] = [
    (
        "rounded",
        [
            "╭─┬╮",
            "│ ││",
            "├─┼┤",
            "│ ││",
            "├─┼┤",
            "├─┼┤",
            "│ ││",
            "╰─┴╯",
        ],
    ),
    (
        "ascii",
        [
            "+--+", "| ||", "|-+|", "| ||", "|-+|", "|-+|", "| ||", "+--+",
        ],
    ),
    (
        "simple",
        [
            "    ", "    ", " ── ", "    ", "    ", " ── ", "    ", "    ",
        ],
    ),
    (
        "heavy",
        [
            "┏━┳┓",
            "┃ ┃┃",
            "┣━╋┫",
            "┃ ┃┃",
            "┣━╋┫",
            "┣━╋┫",
            "┃ ┃┃",
            "┗━┻┛",
        ],
    ),
    (
        "double",
        [
            "╔═╦╗",
            "║ ║║",
            "╠═╬╣",
            "║ ║║",
            "╠═╬╣",
            "╠═╬╣",
            "║ ║║",
            "╚═╩╝",
        ],
    ),
    (
        "minimal",
        [
            "  ╷ ",
            "  │ ",
            "╶─┼╴",
            "  │ ",
            "╶─┼╴",
            "╶─┼╴",
            "  │ ",
            "  ╵ ",
        ],
    ),
    (
        "square",
        [
            "┌─┬┐",
            "│ ││",
            "├─┼┤",
            "│ ││",
            "├─┼┤",
            "├─┼┤",
            "│ ││",
            "└─┴┘",
        ],
    ),
    (
        "heavy_head",
        [
            "┏━┳┓",
            "┃ ┃┃",
            "┡━╇┩",
            "│ ││",
            "├─┼┤",
            "├─┼┤",
            "│ ││",
            "└─┴┘",
        ],
    ),
];

fn rich_box(name: &str) -> [Vec<char>; 8] {
    let b = BOXES
        .iter()
        .find(|(n, _)| *n == name)
        .map_or(&BOXES[0].1, |(_, b)| b);
    std::array::from_fn(|i| b[i].chars().collect())
}

fn s<'a>(v: &'a Value, k: &str) -> Option<&'a str> {
    v.get(k).and_then(Value::as_str)
}

fn pad_to(line: &str, width: usize) -> String {
    format!(
        "{line}{}",
        " ".repeat(width.saturating_sub(vis_width(line)))
    )
}

fn center_painted(text: &str, width: usize, st: &St, d: Depth) -> Vec<String> {
    let cells = parse_markup(text)
        .into_iter()
        .map(|c| Cell {
            ch: c.ch,
            st: st.over(c.st),
        })
        .collect::<Vec<_>>();
    wrap(&cells, width)
        .into_iter()
        .map(|l| {
            let w: usize = l.iter().map(|c| crate::text::cw(c.ch)).sum();
            let left = width.saturating_sub(w) / 2;
            let mut padded = vec![Cell { ch: ' ', st: *st }; left];
            padded.extend(l);
            padded.extend(vec![
                Cell { ch: ' ', st: *st };
                width.saturating_sub(w + left)
            ]);
            spans_to_ansi(&padded, d)
        })
        .collect()
}

pub fn render(node: &Value, width: usize, d: Depth) -> Vec<String> {
    match s(node, "t").unwrap_or("") {
        "text" => {
            let cells = match s(node, "markup") {
                Some(m) => parse_markup(m),
                None => plain_cells(
                    s(node, "plain").unwrap_or(""),
                    parse_style(s(node, "style").unwrap_or("")),
                ),
            };
            render_text(&cells, width, d)
        }
        "table" => render_table(node, width, d),
        "panel" => render_panel(node, width, d),
        "tree" => render_tree(node, d),
        "error" => render_error(node, width, d),
        "progress" => crate::progress::render_frame(node, width, d),
        other => panic!("unknown renderable {other}"),
    }
}

fn cell_ansi(markup: &str, base: St, d: Depth) -> String {
    let cells: Vec<Cell> = parse_markup(markup)
        .into_iter()
        .map(|c| Cell {
            ch: c.ch,
            st: base.over(c.st),
        })
        .collect();
    spans_to_ansi(&cells, d)
}

fn render_table(node: &Value, width: usize, d: Depth) -> Vec<String> {
    let b = rich_box(s(node, "box").unwrap_or("rounded"));
    let show_lines = node
        .get("show_lines")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let line = |i: usize| LineStyle::new(b[i][0], b[i][1], b[i][2], b[i][3]);
    let content = |i: usize| ContentLineStyle::new(b[i][0], b[i][2], b[i][3]);
    let mut style = TableStyle::new()
        .top_border(line(0))
        .header_lines(content(1))
        .header_separator(line(2))
        .content_lines(content(3))
        .bottom_border(line(7));
    if show_lines {
        style = style.row_separator(line(4));
    }
    let cols = node["columns"].as_array().unwrap();
    let align = |c: &Value| match s(c, "justify") {
        Some("right") => CellAlignment::Right,
        Some("center") => CellAlignment::Center,
        _ => CellAlignment::Left,
    };
    let mut table = Table::new();
    table
        .load_style(style)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_width(width as u16);
    let header_st = St {
        bold: true,
        ..St::default()
    };
    table.set_header(
        cols.iter()
            .map(|c| {
                comfy_table::Cell::new(cell_ansi(s(c, "header").unwrap_or(""), header_st, d))
                    .set_alignment(align(c))
            })
            .collect::<Vec<_>>(),
    );
    for row in node["rows"].as_array().unwrap() {
        table.add_row(
            row.as_array()
                .unwrap()
                .iter()
                .zip(cols)
                .map(|(v, c)| {
                    let base = parse_style(s(c, "style").unwrap_or(""));
                    comfy_table::Cell::new(cell_ansi(v.as_str().unwrap_or(""), base, d))
                        .set_alignment(align(c))
                })
                .collect::<Vec<_>>(),
        );
    }
    for (i, c) in cols.iter().enumerate() {
        if c.get("no_wrap").and_then(Value::as_bool) == Some(true) {
            table
                .column_mut(i)
                .unwrap()
                .set_constraint(ColumnConstraint::ContentWidth);
        }
    }
    let body: Vec<String> = table.lines().collect();
    let table_w = body.first().map_or(0, |l| vis_width(l));
    let mut out = Vec::new();
    if let Some(t) = s(node, "title") {
        out.extend(center_painted(
            t,
            table_w,
            &St {
                italic: true,
                ..St::default()
            },
            d,
        ));
    }
    out.extend(body);
    if let Some(c) = s(node, "caption") {
        out.extend(center_painted(
            c,
            table_w,
            &St {
                dim: true,
                italic: true,
                ..St::default()
            },
            d,
        ));
    }
    out
}

fn natural(node: &Value, avail: usize, d: Depth) -> usize {
    match s(node, "t").unwrap_or("") {
        "text" => {
            let cells = match s(node, "markup") {
                Some(m) => parse_markup(m),
                None => plain_cells(s(node, "plain").unwrap_or(""), St::default()),
            };
            natural_width(&cells).min(avail)
        }
        _ => render(node, avail, d)
            .iter()
            .map(|l| vis_width(l))
            .max()
            .unwrap_or(0),
    }
}

fn render_panel(node: &Value, width: usize, d: Depth) -> Vec<String> {
    let b = rich_box(s(node, "box").unwrap_or("rounded"));
    let pad = node
        .get("padding")
        .and_then(Value::as_array)
        .map_or((0, 1), |p| {
            (
                p[0].as_u64().unwrap_or(0) as usize,
                p[1].as_u64().unwrap_or(1) as usize,
            )
        });
    let border = parse_style(s(node, "border_style").unwrap_or(""));
    let expand = node.get("expand").and_then(Value::as_bool).unwrap_or(true);
    let body = &node["body"];
    let inner_avail = width.saturating_sub(2 + 2 * pad.1);
    let panel_w = if expand {
        width
    } else {
        natural(body, inner_avail, d) + 2 * pad.1 + 2
    };
    let inner = panel_w.saturating_sub(2 + 2 * pad.1);
    let mut lines: Vec<String> = Vec::new();
    let blank = " ".repeat(panel_w - 2);
    lines.extend(std::iter::repeat_n(blank.clone(), pad.0));
    for l in render(body, inner, d) {
        lines.push(format!(
            "{}{}{}",
            " ".repeat(pad.1),
            pad_to(&l, inner),
            " ".repeat(pad.1)
        ));
    }
    lines.extend(std::iter::repeat_n(blank, pad.0));
    let p = |t: &str| paint(t, &border, d);
    let edge = |row: &[char], l: char, r: char, title: Option<&str>| -> String {
        let fill = row[1].to_string();
        let Some(t) = title.filter(|_| panel_w > 4) else {
            return p(&format!("{l}{}{r}", fill.repeat(panel_w - 2)));
        };
        let tc: Vec<Cell> = parse_markup(&format!(" {t} "))
            .into_iter()
            .map(|c| Cell {
                ch: c.ch,
                st: border.over(c.st),
            })
            .collect();
        let tw: usize = tc.iter().map(|c| crate::text::cw(c.ch)).sum();
        let room = panel_w - 4;
        let left = room.saturating_sub(tw) / 2;
        let right = room.saturating_sub(tw + left);
        format!(
            "{}{}{}{}{}",
            p(&format!("{l}{}", row[1])),
            p(&fill.repeat(left)),
            spans_to_ansi(&tc, d),
            p(&fill.repeat(right)),
            p(&format!("{}{r}", row[1]))
        )
    };
    let mut out = vec![edge(&b[0], b[0][0], b[0][3], s(node, "title"))];
    for l in lines {
        out.push(format!(
            "{}{l}{}",
            p(&b[3][0].to_string()),
            p(&b[3][3].to_string())
        ));
    }
    out.push(edge(&b[7], b[7][0], b[7][3], s(node, "subtitle")));
    out
}

fn render_tree(node: &Value, d: Depth) -> Vec<String> {
    let guide = parse_style(s(node, "guide_style").unwrap_or(""));
    let heavy = guide.bold;
    let guide = St {
        bold: false,
        ..guide
    };
    let g: [&str; 4] = if heavy {
        ["    ", "┃   ", "┣━━ ", "┗━━ "]
    } else {
        ["    ", "│   ", "├── ", "└── "]
    };
    let mut out = Vec::new();
    let root = &node["root"];
    out.push(spans_to_ansi(
        &parse_markup(s(root, "label").unwrap_or("")),
        d,
    ));
    fn walk(
        n: &Value,
        prefix: &[String],
        guide: &St,
        g: &[&str; 4],
        d: Depth,
        out: &mut Vec<String>,
    ) {
        let kids = n["children"].as_array().map_or(&[][..], |v| v.as_slice());
        for (i, k) in kids.iter().enumerate() {
            let last = i + 1 == kids.len();
            let fork = paint(if last { g[3] } else { g[2] }, guide, d);
            out.push(format!(
                "{}{fork}{}",
                prefix.concat(),
                spans_to_ansi(&parse_markup(s(k, "label").unwrap_or("")), d)
            ));
            let mut next = prefix.to_vec();
            next.push(paint(if last { g[0] } else { g[1] }, guide, d));
            walk(k, &next, guide, g, d, out);
        }
    }
    walk(root, &[], &guide, &g, d, &mut out);
    out
}

fn render_error(node: &Value, width: usize, d: Depth) -> Vec<String> {
    let mut lines = vec![Value::String(String::new()); 0];
    let text = |plain: String, style: &str| serde_json::json!({"t": "text", "plain": plain, "style": style});
    lines.push(text(s(node, "message").unwrap_or("").into(), "bold"));
    let causes = node["causes"].as_array().cloned().unwrap_or_default();
    if !causes.is_empty() {
        lines.push(text(String::new(), ""));
        lines.push(text("Caused by:".into(), "dim"));
        for (i, c) in causes.iter().enumerate() {
            lines.push(text(format!("    {i}: {}", c.as_str().unwrap_or("")), ""));
        }
    }
    if let Some(h) = s(node, "hint") {
        lines.push(text(String::new(), ""));
        lines.push(text(format!("hint: {h}"), "cyan"));
    }
    let inner = width.saturating_sub(4);
    let body: Vec<String> = lines.iter().flat_map(|l| render(l, inner, d)).collect();
    let panel = serde_json::json!({
        "t": "panel", "title": "Error", "box": "rounded", "border_style": "red", "expand": true,
        "padding": [0, 1], "body": {"t": "text", "plain": "", "style": ""}
    });
    let shell = render_panel(&panel, width, d);
    let mut out = vec![shell[0].clone()];
    let p = |t: &str| paint(t, &parse_style("red"), d);
    for l in body {
        out.push(format!("{} {} {}", p("│"), pad_to(&l, inner), p("│")));
    }
    out.push(shell[shell.len() - 1].clone());
    out
}
