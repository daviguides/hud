//! Case renderer for the hud bench: declarative case (bench/spec/case-schema.md) to bytes, through rich-rs.

use rich_rs::r#box as rbox;
use rich_rs::{
    BarColumn, ColorSystem, Column, Console, ConsoleOptions, JustifyMethod, LiveOptions,
    MofNCompleteColumn, Panel, Progress, ProgressColumn, Renderable, Row, Style, Table,
    TaskProgressColumn, Text, TextColumn, Tree,
};
use serde_json::Value;

type Node = Box<dyn Renderable + Send + Sync>;

/// Variant switch: by default the adapter applies the Rich default-theme styling the schema leaves implicit
/// (table title italic, caption italic dim) through the crate's own `with_title_style`/`with_caption_style`.
/// `HUD_BENCH_CRATE_DEFAULTS=1` leaves the crate defaults untouched.
fn theme_parity() -> bool {
    std::env::var_os("HUD_BENCH_CRATE_DEFAULTS").is_none()
}

pub fn style(s: &str) -> Style {
    Style::parse(s).unwrap_or_default()
}

fn str_of<'a>(v: &'a Value, key: &str) -> &'a str {
    v.get(key).and_then(Value::as_str).unwrap_or("")
}

pub fn markup(s: &str) -> Result<Text, String> {
    Text::from_markup(s, false).map_err(|e| format!("{e:?}"))
}

fn box_of(name: &str) -> rbox::Box {
    match name {
        "ascii" => rbox::ASCII,
        "simple" => rbox::SIMPLE,
        "heavy" => rbox::HEAVY,
        "double" => rbox::DOUBLE,
        "minimal" => rbox::MINIMAL,
        "square" => rbox::SQUARE,
        "heavy_head" => rbox::HEAVY_HEAD,
        _ => rbox::ROUNDED,
    }
}

fn justify_of(name: &str) -> JustifyMethod {
    match name {
        "center" => JustifyMethod::Center,
        "right" => JustifyMethod::Right,
        _ => JustifyMethod::Left,
    }
}

fn add_children(parent: &mut Tree, children: &[Value]) -> Result<(), String> {
    for child in children {
        let label = markup(str_of(child, "label"))?;
        let sub = parent.add(Box::new(label));
        add_children(sub, child["children"].as_array().map(Vec::as_slice).unwrap_or(&[]))?;
    }
    Ok(())
}

fn error_body(node: &Value) -> Text {
    let mut body = Text::new();
    body.append(str_of(node, "message"), Some(style("bold")));
    let causes = node["causes"].as_array().map(Vec::as_slice).unwrap_or(&[]);
    if !causes.is_empty() {
        body.append("\n\n", None);
        body.append("Caused by:", Some(style("dim")));
        for (i, cause) in causes.iter().enumerate() {
            body.append(format!("\n    {i}: {}", cause.as_str().unwrap_or("")), None);
        }
    }
    if let Some(hint) = node.get("hint").and_then(Value::as_str) {
        body.append("\n\n", None);
        body.append(format!("hint: {hint}"), Some(style("cyan")));
    }
    body
}

pub fn build(node: &Value) -> Result<Node, String> {
    Ok(match str_of(node, "t") {
        "text" => {
            if let Some(m) = node.get("markup").and_then(Value::as_str) {
                Box::new(markup(m)?)
            } else {
                Box::new(Text::styled(str_of(node, "plain"), style(str_of(node, "style"))))
            }
        }
        "table" => {
            let mut table = Table::new()
                .with_box(Some(box_of(str_of(node, "box"))))
                .with_show_lines(node["show_lines"].as_bool().unwrap_or(false));
            if let Some(title) = node["title"].as_str() {
                table = table.with_title(title);
                if theme_parity() {
                    table = table.with_title_style(style("italic"));
                }
            }
            if let Some(caption) = node["caption"].as_str() {
                table = table.with_caption(caption);
                if theme_parity() {
                    table = table.with_caption_style(style("italic dim"));
                }
            }
            for c in node["columns"].as_array().into_iter().flatten() {
                let mut col = Column::with_header_str(str_of(c, "header"))
                    .justify(justify_of(str_of(c, "justify")));
                if let Some(s) = c.get("style").and_then(Value::as_str) {
                    col = col.style(style(s));
                }
                if c.get("no_wrap").and_then(Value::as_bool).unwrap_or(false) {
                    col = col.no_wrap(true);
                }
                table.add_column(col);
            }
            for row in node["rows"].as_array().into_iter().flatten() {
                let cells = row
                    .as_array()
                    .into_iter()
                    .flatten()
                    .map(|c| markup(c.as_str().unwrap_or("")).map(|t| Box::new(t) as Node))
                    .collect::<Result<Vec<_>, _>>()?;
                table.add_row(Row::new(cells));
            }
            Box::new(table)
        }
        "panel" => {
            let pad = node["padding"].as_array();
            let (v, h) = pad
                .map(|p| (p[0].as_u64().unwrap_or(0) as usize, p[1].as_u64().unwrap_or(1) as usize))
                .unwrap_or((0, 1));
            let mut panel = Panel::new(build(&node["body"])?)
                .with_box(box_of(str_of(node, "box")))
                .with_expand(node["expand"].as_bool().unwrap_or(true))
                .with_padding((v, h))
                .with_border_style(style(str_of(node, "border_style")));
            if let Some(t) = node["title"].as_str() {
                panel = panel.with_title(t);
            }
            if let Some(t) = node["subtitle"].as_str() {
                panel = panel.with_subtitle(t);
            }
            Box::new(panel)
        }
        "tree" => {
            let root = &node["root"];
            let mut tree = Tree::new(Box::new(markup(str_of(root, "label"))?))
                .with_guide_style(style(str_of(node, "guide_style")));
            add_children(&mut tree, root["children"].as_array().map(Vec::as_slice).unwrap_or(&[]))?;
            Box::new(tree)
        }
        "progress" => {
            let columns: Vec<Box<dyn ProgressColumn>> = vec![
                Box::new(TextColumn::new("{task.description}")),
                Box::new(BarColumn::new().with_bar_width(Some(
                    node["bar_width"].as_u64().unwrap_or(40) as usize,
                ))),
                Box::new(TaskProgressColumn::new(false)),
                Box::new(MofNCompleteColumn::new()),
            ];
            let live = LiveOptions { auto_refresh: false, ..LiveOptions::default() };
            let progress = Progress::new(columns, live, false, false);
            for t in node["tasks"].as_array().into_iter().flatten() {
                progress.add_task(
                    str_of(t, "description"),
                    true,
                    t["total"].as_f64(),
                    t["completed"].as_f64().unwrap_or(0.0),
                    true,
                );
            }
            Box::new(progress)
        }
        "error" => Box::new(
            Panel::new(Box::new(error_body(node)))
                .with_title("Error")
                .with_box(rbox::ROUNDED)
                .with_border_style(style("red")),
        ),
        other => return Err(format!("unknown renderable {other}")),
    })
}

pub fn case_console(width: usize, color: &str) -> Console<Vec<u8>> {
    let color_system = match color {
        "truecolor" => Some(ColorSystem::TrueColor),
        "256" => Some(ColorSystem::EightBit),
        "standard" => Some(ColorSystem::Standard),
        _ => None,
    };
    Console::capture_with_options(ConsoleOptions {
        size: (width, 24),
        max_width: width,
        max_height: 24,
        is_terminal: true,
        emoji_enabled: false,
        highlight_enabled: false,
        color_system,
        ..ConsoleOptions::default()
    })
}

pub fn render_case(case: &Value) -> Result<Vec<u8>, String> {
    let node = &case["renderable"];
    let renderable = build(node)?;
    let mut console = case_console(
        case["width"].as_u64().unwrap_or(80) as usize,
        str_of(case, "color_system"),
    );
    let end = if str_of(node, "t") == "text" { "\n" } else { "" };
    console
        .print(renderable.as_ref(), None, None, None, false, end)
        .map_err(|e| e.to_string())?;
    Ok(console.get_captured_bytes().to_vec())
}
