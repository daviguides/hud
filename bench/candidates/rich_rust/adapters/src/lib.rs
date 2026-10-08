use std::io::Write;
use std::sync::{Arc, Mutex};

use rich_rust::r#box::{ASCII, BoxChars, DOUBLE, HEAVY, HEAVY_HEAD, MINIMAL, ROUNDED, SIMPLE, SQUARE};
use rich_rust::console::PrintOptions;
use rich_rust::markup;
use rich_rust::prelude::*;
use rich_rust::renderables::Renderable;
use rich_rust::segment::split_lines;
use serde_json::Value;

pub struct SharedBuf(pub Arc<Mutex<Vec<u8>>>);

impl Write for SharedBuf {
    fn write(&mut self, data: &[u8]) -> std::io::Result<usize> {
        self.0.lock().unwrap().extend_from_slice(data);
        Ok(data.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

pub type Captured = Arc<Mutex<Vec<u8>>>;

/// Console configured as `cases/case-schema.md` asks: fixed width, terminal on,
/// explicit color depth, no emoji codes, no highlighting. `none` has no color
/// system and is not a terminal: the crate cannot express "terminal on without
/// a color system" (see NOTES.md).
pub fn case_console(width: usize, color_system: &str) -> (Console, Captured) {
    let captured: Captured = Arc::new(Mutex::new(Vec::new()));
    let builder = Console::builder()
        .width(width)
        .height(24)
        .emoji(false)
        .highlight(false)
        .file(Box::new(SharedBuf(captured.clone())));
    let builder = match color_system {
        "truecolor" => builder.force_terminal(true).color_system(ColorSystem::TrueColor),
        "256" => builder.force_terminal(true).color_system(ColorSystem::EightBit),
        "standard" => builder.force_terminal(true).color_system(ColorSystem::Standard),
        _ => builder.force_terminal(false).no_color(),
    };
    (builder.build(), captured)
}

fn box_chars(name: &str) -> &'static BoxChars {
    match name {
        "ascii" => &ASCII,
        "simple" => &SIMPLE,
        "heavy" => &HEAVY,
        "double" => &DOUBLE,
        "minimal" => &MINIMAL,
        "square" => &SQUARE,
        "heavy_head" => &HEAVY_HEAD,
        _ => &ROUNDED,
    }
}

fn style_of(spec: &str) -> Result<Style, String> {
    if spec.trim().is_empty() {
        return Ok(Style::new());
    }
    Style::parse(spec).map_err(|e| format!("style {spec:?}: {e:?}"))
}

fn text_of(markup_str: &str) -> Text {
    markup::render_or_plain(markup_str)
}

fn owned_lines(texts: &[Text]) -> Vec<Vec<Segment<'static>>> {
    texts
        .iter()
        .map(|line| line.render("").into_iter().map(Segment::into_owned).collect())
        .collect()
}

fn str_field<'a>(node: &'a Value, key: &str) -> Option<&'a str> {
    node.get(key).and_then(Value::as_str)
}

fn build_table(node: &Value) -> Result<Table, String> {
    let mut table = Table::new()
        .box_style(box_chars(str_field(node, "box").unwrap_or("rounded")))
        .show_lines(node["show_lines"].as_bool().unwrap_or(false));
    if let Some(title) = str_field(node, "title") {
        table = table.title(text_of(title));
    }
    if let Some(caption) = str_field(node, "caption") {
        table = table.caption(text_of(caption));
    }
    for col in node["columns"].as_array().ok_or("columns")? {
        let mut column = Column::new(text_of(str_field(col, "header").unwrap_or("")));
        column = column.justify(match str_field(col, "justify") {
            Some("center") => JustifyMethod::Center,
            Some("right") => JustifyMethod::Right,
            _ => JustifyMethod::Left,
        });
        if let Some(style) = str_field(col, "style") {
            column = column.style(style_of(style)?);
        }
        if col["no_wrap"].as_bool().unwrap_or(false) {
            column = column.no_wrap();
        }
        table.add_column(column);
    }
    for row in node["rows"].as_array().ok_or("rows")? {
        let cells: Vec<&str> = row.as_array().ok_or("row")?.iter().map(|c| c.as_str().unwrap_or("")).collect();
        table.add_row_markup(cells);
    }
    Ok(table)
}

fn render_lines(console: &Console, renderable: &impl Renderable, width: usize) -> Vec<Vec<Segment<'static>>> {
    let options = console.options().update_width(width);
    let segments: Vec<Segment<'static>> =
        renderable.render(console, &options).into_iter().map(Segment::into_owned).collect();
    split_lines(segments.into_iter())
}

fn body_lines(console: &Console, body: &Value, width: usize) -> Result<Vec<Vec<Segment<'static>>>, String> {
    match str_field(body, "t") {
        Some("text") => {
            let text = match str_field(body, "markup") {
                Some(m) => text_of(m),
                None => Text::styled(str_field(body, "plain").unwrap_or(""), style_of(str_field(body, "style").unwrap_or(""))?),
            };
            Ok(owned_lines(&text.wrap(width)))
        }
        Some("table") => Ok(render_lines(console, &build_table(body)?, width)),
        Some("panel") => {
            let outer = panel_of(console, body, width)?;
            Ok(render_lines(console, &outer, width))
        }
        other => Err(format!("body {other:?}")),
    }
}

fn panel_of(console: &Console, node: &Value, width: usize) -> Result<Panel<'static>, String> {
    let pad = node["padding"].as_array().ok_or("padding")?;
    let (vertical, horizontal) = (pad[0].as_u64().unwrap_or(0) as usize, pad[1].as_u64().unwrap_or(0) as usize);
    let inner = width.saturating_sub(2 + 2 * horizontal);
    let mut panel = Panel::new(body_lines(console, &node["body"], inner)?)
        .box_style(box_chars(str_field(node, "box").unwrap_or("rounded")))
        .expand(node["expand"].as_bool().unwrap_or(true))
        .padding((vertical, horizontal))
        .border_style(style_of(str_field(node, "border_style").unwrap_or(""))?);
    if let Some(title) = str_field(node, "title") {
        panel = panel.title(text_of(title));
    }
    if let Some(subtitle) = str_field(node, "subtitle") {
        panel = panel.subtitle(text_of(subtitle));
    }
    Ok(panel)
}

fn tree_node(node: &Value) -> TreeNode {
    let mut out = TreeNode::new(text_of(str_field(node, "label").unwrap_or("")));
    for child in node["children"].as_array().into_iter().flatten() {
        out = out.child(tree_node(child));
    }
    out
}

pub struct Frame {
    pub rows: Vec<(String, u64, u64)>,
    pub bar_width: usize,
    pub bar_complete: Style,
    pub bar_back: Style,
    pub bar_pulse: Style,
}

impl Renderable for Frame {
    fn render<'a>(&'a self, _console: &Console, options: &ConsoleOptions) -> Vec<Segment<'a>> {
        let name_width = self.rows.iter().map(|r| rich_rust::cells::cell_len(&r.0)).max().unwrap_or(0);
        let mut segments = Vec::new();
        for (name, total, done) in &self.rows {
            let mut bar = ProgressBar::with_total(*total)
                .width(self.bar_width)
                .bar_style(BarStyle::Line)
                .show_brackets(false)
                .completed_style(self.bar_complete.clone())
                .remaining_style(self.bar_back.clone())
                .pulse_style(self.bar_pulse.clone())
                .description(format!("{name:<name_width$}"));
            bar.update(*done);
            let mut line = bar.render(options.max_width);
            line.pop();
            let width = total.to_string().len();
            line.push(Segment::plain(format!(" {done:>width$}/{total}")));
            line.push(Segment::line());
            segments.extend(line);
        }
        segments
    }
}

fn error_panel(console: &Console, node: &Value, width: usize) -> Panel<'static> {
    let mut lines: Vec<Text> = vec![Text::styled(str_field(node, "message").unwrap_or(""), Style::new().bold())];
    let causes = node["causes"].as_array().cloned().unwrap_or_default();
    if !causes.is_empty() {
        lines.push(Text::new(""));
        lines.push(Text::styled("Caused by:", Style::new().dim()));
        for (i, cause) in causes.iter().enumerate() {
            lines.push(Text::new(format!("    {i}: {}", cause.as_str().unwrap_or(""))));
        }
    }
    if let Some(hint) = str_field(node, "hint") {
        lines.push(Text::new(""));
        lines.push(Text::styled(format!("hint: {hint}"), console.get_style("cyan")));
    }
    let wrapped: Vec<Text> = lines.iter().flat_map(|line| line.wrap(width.saturating_sub(4))).collect();
    Panel::new(owned_lines(&wrapped))
        .title("Error")
        .border_style(Style::parse("red").unwrap_or_default())
}

/// Render one case. `Err` means the crate cannot express it (unsupported marker).
pub fn render_case(case: &Value) -> Result<Vec<u8>, String> {
    let width = case["width"].as_u64().ok_or("width")? as usize;
    let (console, captured) = case_console(width, str_field(case, "color_system").unwrap_or("none"));
    let node = &case["renderable"];
    match str_field(node, "t") {
        Some("text") => {
            let content = match str_field(node, "markup") {
                Some(m) => m.to_string(),
                None => match str_field(node, "style") {
                    Some(style) => format!("[{style}]{}[/]", str_field(node, "plain").unwrap_or("")),
                    None => str_field(node, "plain").unwrap_or("").to_string(),
                },
            };
            if std::env::var_os("HUD_PRINT_DEFAULT").is_some() {
                console.print(&content);
            } else {
                console.print_with_options(&content, &PrintOptions::new().with_width(width));
            }
        }
        Some("table") => console.print_renderable(&build_table(node)?),
        Some("panel") => console.print_renderable(&panel_of(&console, node, width)?),
        Some("tree") => {
            let guide_style = str_field(node, "guide_style").unwrap_or("");
            let guides = if guide_style.split_whitespace().any(|w| w == "bold") {
                TreeGuides::Bold
            } else {
                TreeGuides::Unicode
            };
            let tree = Tree::new(tree_node(&node["root"])).guides(guides).guide_style(style_of(guide_style)?);
            console.print_renderable(&tree);
        }
        Some("progress") => {
            let rows = node["tasks"]
                .as_array()
                .ok_or("tasks")?
                .iter()
                .map(|t| {
                    (
                        str_field(t, "description").unwrap_or("").to_string(),
                        t["total"].as_u64().unwrap_or(0),
                        t["completed"].as_u64().unwrap_or(0),
                    )
                })
                .collect();
            console.print_renderable(&Frame {
                rows,
                bar_width: node["bar_width"].as_u64().unwrap_or(30) as usize,
                bar_complete: console.get_style("bar.complete"),
                bar_back: console.get_style("bar.back"),
                bar_pulse: console.get_style("bar.pulse"),
            });
        }
        Some("error") => console.print_renderable(&error_panel(&console, node, width)),
        other => return Err(format!("renderable {other:?}")),
    }
    let bytes = captured.lock().unwrap().clone();
    Ok(bytes)
}
