// cases-runner <cases.jsonl> <outdir>
// One <id>.ansi per case hud implements; an empty <id>.unsupported for the rest. hud claims the
// features listed in support.json: every milestone adds to it, and an unclaimed feature is
// unsupported, never a pass.
use std::fs;
use std::path::PathBuf;

use hud::{BoxStyle, Column, ColorSystem, Console, Justify, Padding, Panel, Style, Table, Text, Tree};
use serde_json::{Value, json};

const CLAIMED: &[&str] = &["style", "markup", "table", "panel", "tree"];

fn color_system(name: &str) -> ColorSystem {
    match name {
        "truecolor" => ColorSystem::TrueColor,
        "256" => ColorSystem::EightBit,
        "standard" => ColorSystem::Standard,
        "none" => ColorSystem::None,
        other => panic!("unknown color system {other}"),
    }
}

fn box_style(name: &str) -> Option<BoxStyle> {
    Some(match name {
        "rounded" => BoxStyle::Rounded,
        "ascii" => BoxStyle::Ascii,
        "simple" => BoxStyle::Simple,
        "heavy" => BoxStyle::Heavy,
        "double" => BoxStyle::Double,
        "minimal" => BoxStyle::Minimal,
        "square" => BoxStyle::Square,
        "heavy_head" => BoxStyle::HeavyHead,
        _ => return None,
    })
}

fn justify(name: &str) -> Justify {
    match name {
        "center" => Justify::Center,
        "right" => Justify::Right,
        _ => Justify::Left,
    }
}

fn table(node: &Value) -> Option<Table> {
    let mut table = Table::new()
        .box_style(box_style(node["box"].as_str()?)?)
        .show_lines(node["show_lines"].as_bool().unwrap_or(false));
    if let Some(title) = node["title"].as_str() {
        table = table.title(title);
    }
    if let Some(caption) = node["caption"].as_str() {
        table = table.caption(caption);
    }
    for column in node["columns"].as_array()? {
        let mut built = Column::new(column["header"].as_str()?)
            .justify(justify(column["justify"].as_str().unwrap_or("left")))
            .no_wrap(column["no_wrap"].as_bool().unwrap_or(false));
        if let Some(style) = column["style"].as_str().filter(|s| !s.is_empty()) {
            built = built.style(Style::parse(style).ok()?);
        }
        table.add_column(built);
    }
    for row in node["rows"].as_array()? {
        table.add_row(row.as_array()?.iter().map(|cell| cell.as_str().unwrap_or("")));
    }
    Some(table)
}

fn style(definition: &str) -> Option<Style> {
    Style::parse(definition).ok()
}

fn panel(node: &Value) -> Option<Panel> {
    let body = &node["body"];
    let built = match body["t"].as_str()? {
        "text" => Panel::new(body["markup"].as_str()?),
        "table" => Panel::new(table(body)?),
        "panel" => Panel::new(panel(body)?),
        _ => return None,
    };
    let pad = node["padding"].as_array()?;
    let mut built = built
        .box_style(box_style(node["box"].as_str()?)?)
        .expand(node["expand"].as_bool()?)
        .padding(Padding::from((pad.first()?.as_u64()? as usize, pad.get(1)?.as_u64()? as usize)));
    if let Some(title) = node["title"].as_str() {
        built = built.title(title);
    }
    if let Some(subtitle) = node["subtitle"].as_str() {
        built = built.subtitle(subtitle);
    }
    if let Some(border) = node["border_style"].as_str().filter(|s| !s.is_empty()) {
        built = built.border_style(style(border)?);
    }
    Some(built)
}

fn tree_node(node: &Value) -> Option<Tree> {
    let mut built = Tree::new(node["label"].as_str()?);
    for child in node["children"].as_array()? {
        built = built.child(tree_node(child)?);
    }
    Some(built)
}

fn tree(node: &Value) -> Option<Tree> {
    let mut root = tree_node(&node["root"])?;
    if let Some(guide) = node["guide_style"].as_str().filter(|s| !s.is_empty()) {
        root = root.guide_style(style(guide)?);
    }
    Some(root)
}

fn render(case: &Value) -> Option<String> {
    let node = &case["renderable"];
    let kind = node["t"].as_str()?;
    if !matches!(kind, "text" | "table" | "panel" | "tree") {
        return None;
    }
    let system = color_system(case["color_system"].as_str()?);
    let console = Console::builder()
        .width(case["width"].as_u64()? as u16)
        .height(24)
        .color_system(system)
        .attributes(system != ColorSystem::None)
        .build();
    match kind {
        "table" => return Some(console.render_to_string(&table(node)?)),
        "panel" => return Some(console.render_to_string(&panel(node)?)),
        "tree" => return Some(console.render_to_string(&tree(node)?)),
        _ => {}
    }
    let text = match node["markup"].as_str() {
        Some(markup) => Text::from_markup(markup).ok()?,
        None => Text::styled(
            node["plain"].as_str()?,
            Style::parse(node["style"].as_str().unwrap_or("")).ok()?,
        ),
    };
    Some(console.render_to_string(&text))
}

fn main() {
    let mut args = std::env::args().skip(1);
    let cases = PathBuf::from(args.next().expect("cases.jsonl"));
    let outdir = PathBuf::from(args.next().expect("outdir"));
    fs::create_dir_all(&outdir).unwrap();
    for line in fs::read_to_string(cases).unwrap().lines().filter(|l| !l.trim().is_empty()) {
        let case: Value = serde_json::from_str(line).unwrap();
        let id = case["id"].as_str().unwrap();
        let feature = case["feature"].as_str().unwrap();
        match CLAIMED.contains(&feature).then(|| render(&case)).flatten() {
            Some(out) => fs::write(outdir.join(format!("{id}.ansi")), out).unwrap(),
            None => fs::write(outdir.join(format!("{id}.unsupported")), "").unwrap(),
        }
    }
    fs::write(
        outdir.join("support.json"),
        json!({ "features": CLAIMED }).to_string(),
    )
    .unwrap();
}
