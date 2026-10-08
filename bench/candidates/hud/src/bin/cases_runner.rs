// cases-runner <cases.jsonl> <outdir>
// One <id>.ansi per case hud implements; an empty <id>.unsupported for the rest. hud claims the
// features listed in support.json: every milestone adds to it, and an unclaimed feature is
// unsupported, never a pass.
use std::fs;
use std::path::PathBuf;

use hud::{
    BarColumn, BoxStyle, ColorSystem, Column, Console, Justify, MofNCompleteColumn, Padding, Panel,
    Progress, Style, Table, TaskProgressColumn, Text, TextColumn, Tree,
};
use serde_json::{Value, json};

const CLAIMED: &[&str] = &["style", "markup", "table", "panel", "tree", "progress"];

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

fn progress(node: &Value, console: &Console) -> Option<Progress> {
    let progress = Progress::builder()
        .console(console.clone())
        .column(TextColumn::new("{task.description}"))
        .column(BarColumn::new().bar_width(node["bar_width"].as_u64()? as usize))
        .column(TaskProgressColumn::new())
        .column(MofNCompleteColumn::new())
        .disable(true)
        .build();
    for task in node["tasks"].as_array()? {
        let added = progress.add_task(task["description"].as_str()?, task["total"].as_u64()?);
        added.set_completed(task["completed"].as_u64()?);
    }
    Some(progress)
}

fn render(case: &Value) -> Option<String> {
    let node = &case["renderable"];
    let kind = node["t"].as_str()?;
    if !matches!(kind, "text" | "table" | "panel" | "tree" | "progress") {
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
        "progress" => return Some(console.render_to_string(&progress(node, &console)?)),
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

// cases-runner --widgets <table_unicode.jsonl> <outdir>
// Every Unicode table case as the body of an expanding and of a fitting panel with a wide title
// and subtitle, and as the labels of a tree: panels/<id>-expand.ansi, panels/<id>-fit.ansi and
// trees/<id>.ansi, for width_check.py (no row wider than the terminal, panel rows alike).
fn widgets(cases: PathBuf, outdir: PathBuf) {
    fs::create_dir_all(outdir.join("panels")).unwrap();
    fs::create_dir_all(outdir.join("trees")).unwrap();
    for line in fs::read_to_string(cases).unwrap().lines().filter(|l| !l.trim().is_empty()) {
        let case: Value = serde_json::from_str(line).unwrap();
        let id = case["id"].as_str().unwrap();
        let node = &case["renderable"];
        let system = color_system(case["color_system"].as_str().unwrap());
        let console = Console::builder()
            .width(case["width"].as_u64().unwrap() as u16)
            .height(24)
            .color_system(system)
            .attributes(system != ColorSystem::None)
            .build();
        let table = table(node).unwrap();
        for (name, fit) in [("expand", false), ("fit", true)] {
            let panel = Panel::new(table.clone())
                .title("日本語 ✓ 😀")
                .subtitle("é ＡＢＣ")
                .expand(!fit);
            fs::write(
                outdir.join("panels").join(format!("{id}-{name}.ansi")),
                console.render_to_string(&panel),
            )
            .unwrap();
        }
        let mut tree = Tree::new("日本語 root");
        for column in node["columns"].as_array().unwrap() {
            let header = column["header"].as_str().unwrap();
            let branch = tree.add(header);
            for row in node["rows"].as_array().unwrap() {
                for cell in row.as_array().unwrap() {
                    branch.add(cell.as_str().unwrap());
                }
            }
        }
        fs::write(
            outdir.join("trees").join(format!("{id}.ansi")),
            console.render_to_string(&tree),
        )
        .unwrap();
    }
}

fn main() {
    let mut args = std::env::args().skip(1).peekable();
    if args.peek().is_some_and(|a| a == "--widgets") {
        args.next();
        let cases = PathBuf::from(args.next().expect("table_unicode.jsonl"));
        widgets(cases, PathBuf::from(args.next().expect("outdir")));
        return;
    }
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
