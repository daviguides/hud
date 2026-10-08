// live-layout-runner <live_layout.jsonl> <outdir>
// Corpus 3: one <id>.ansi per Columns, Layout and Live case, written like cases_runner.rs writes
// the base corpus. The node builders below repeat the ones of cases_runner.rs on purpose: that file
// belongs to the main line and this adapter must not touch it.
use std::fs;
use std::path::PathBuf;

use hud::{
    Align, Body, BoxStyle, ColorSystem, Column, Columns, Console, Justify, Layout, Padding, Panel,
    Style, Table, Text, Tree,
};
use serde_json::{Value, json};

const CLAIMED: &[&str] = &["columns", "layout"];

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

fn style(definition: &str) -> Option<Style> {
    Style::parse(definition).ok()
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
        if let Some(s) = column["style"].as_str().filter(|s| !s.is_empty()) {
            built = built.style(style(s)?);
        }
        table.add_column(built);
    }
    for row in node["rows"].as_array()? {
        table.add_row(row.as_array()?.iter().map(|cell| cell.as_str().unwrap_or("")));
    }
    Some(table)
}

fn panel(node: &Value) -> Option<Panel> {
    let pad = node["padding"].as_array()?;
    let mut built = Panel::new(build(&node["body"])?)
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

fn padding_of(values: &[Value]) -> Option<Padding> {
    let number = |index: usize| values.get(index)?.as_u64().map(|n| n as usize);
    match values.len() {
        1 => Some(Padding::from(number(0)?)),
        2 => Some(Padding::from((number(0)?, number(1)?))),
        4 => Some(Padding::from((number(0)?, number(1)?, number(2)?, number(3)?))),
        _ => None,
    }
}

fn columns(node: &Value) -> Option<Columns> {
    let mut items = Vec::new();
    for item in node["items"].as_array()? {
        items.push(build(item)?);
    }
    let mut built = Columns::new(items)
        .expand(node["expand"].as_bool().unwrap_or(false))
        .equal(node["equal"].as_bool().unwrap_or(false))
        .column_first(node["column_first"].as_bool().unwrap_or(false))
        .right_to_left(node["right_to_left"].as_bool().unwrap_or(false));
    if let Some(values) = node["padding"].as_array() {
        built = built.padding(padding_of(values)?);
    }
    if let Some(width) = node["width"].as_u64() {
        built = built.width(width as usize);
    }
    built = match node["align"].as_str() {
        Some("left") => built.align(Align::Left),
        Some("center") => built.align(Align::Center),
        Some("right") => built.align(Align::Right),
        _ => built,
    };
    if let Some(title) = node["title"].as_str() {
        built = built.title(title);
    }
    Some(built)
}

fn layout(node: &Value) -> Option<Layout> {
    let mut built = if let Some(children) = node["children"].as_array() {
        let mut kids = Vec::new();
        for child in children {
            kids.push(layout(child)?);
        }
        match node["splitter"].as_str()? {
            "row" => Layout::row(kids),
            _ => Layout::column(kids),
        }
    } else {
        Layout::new(build(&node["renderable"])?)
    };
    if let Some(name) = node["name"].as_str() {
        built = built.name(name);
    }
    if let Some(size) = node["size"].as_u64() {
        built = built.size(size as usize);
    }
    if let Some(minimum) = node["minimum_size"].as_u64() {
        built = built.minimum_size(minimum as usize);
    }
    if let Some(ratio) = node["ratio"].as_u64() {
        built = built.ratio(ratio as usize);
    }
    if let Some(visible) = node["visible"].as_bool() {
        built = built.visible(visible);
    }
    Some(built)
}

/// A node of the schema as something a grid, a panel, a layout or a live display can hold.
fn build(node: &Value) -> Option<Body> {
    Some(match node["t"].as_str()? {
        "text" => Body::from(Text::from_markup(node["markup"].as_str()?).ok()?),
        "table" => Body::from(table(node)?),
        "panel" => Body::from(panel(node)?),
        "tree" => Body::from(tree(node)?),
        "columns" => Body::from(columns(node)?),
        "layout" => Body::from(layout(&node["root"])?),
        _ => return None,
    })
}

fn console_for(case: &Value) -> Option<Console> {
    let system = color_system(case["color_system"].as_str()?);
    Some(
        Console::builder()
            .width(case["width"].as_u64()? as u16)
            .height(case["height"].as_u64().unwrap_or(24) as u16)
            .color_system(system)
            .attributes(system != ColorSystem::None)
            .build(),
    )
}

fn render(case: &Value) -> Option<String> {
    let node = &case["renderable"];
    let console = console_for(case)?;
    match node["t"].as_str()? {
        "columns" => Some(console.render_to_string(&columns(node)?)),
        "layout" => Some(console.render_to_string(&layout(&node["root"])?)),
        _ => None,
    }
}

fn main() {
    let mut args = std::env::args().skip(1);
    let cases = PathBuf::from(args.next().expect("live_layout.jsonl"));
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
    fs::write(outdir.join("support.json"), json!({ "features": CLAIMED }).to_string()).unwrap();
}
