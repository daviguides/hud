// cases-runner <cases.jsonl> <outdir>
// One <id>.ansi per case; an empty <id>.unsupported when the crate cannot build the case.
use std::fs;
use std::path::PathBuf;

use rich::r#box::{self, Box as BoxSet};
use rich::{
    BarColumn, ColorSystem, Console, Justify, Panel, Progress, ProgressColumn, Renderable, Style,
    Table, Text, TextColumn, Tree,
};
use serde_json::Value;

fn s<'a>(v: &'a Value, key: &str) -> Option<&'a str> {
    v.get(key).and_then(Value::as_str)
}

fn box_set(name: &str) -> BoxSet {
    match name {
        "rounded" => r#box::ROUNDED,
        "ascii" => r#box::ASCII,
        "simple" => r#box::SIMPLE,
        "heavy" => r#box::HEAVY,
        "double" => r#box::DOUBLE,
        "minimal" => r#box::MINIMAL,
        "square" => r#box::SQUARE,
        "heavy_head" => r#box::HEAVY_HEAD,
        other => panic!("unknown box {other}"),
    }
}

fn justify(name: &str) -> Justify {
    match name {
        "left" => Justify::Left,
        "center" => Justify::Center,
        "right" => Justify::Right,
        other => panic!("unknown justify {other}"),
    }
}

fn add_children(parent: &mut Tree, children: &[Value]) {
    for child in children {
        let node = parent.add(s(child, "label").unwrap());
        add_children(node, child["children"].as_array().unwrap());
    }
}

fn build(node: &Value) -> Result<Box<dyn Renderable>, String> {
    match s(node, "t").unwrap() {
        "text" => {
            if let Some(markup) = s(node, "markup") {
                let text = Text::from_markup(markup).map_err(|e| format!("{e:?}"))?;
                Ok(Box::new(text))
            } else {
                Ok(Box::new(Text::styled(
                    s(node, "plain").unwrap(),
                    s(node, "style").unwrap_or(""),
                )))
            }
        }
        "table" => {
            let mut table = Table::new()
                .box_set(box_set(s(node, "box").unwrap()))
                .show_lines(node["show_lines"].as_bool().unwrap_or(false));
            if let Some(title) = s(node, "title") {
                table = table.title(title);
            }
            if let Some(caption) = s(node, "caption") {
                table = table.caption(caption);
            }
            for column in node["columns"].as_array().unwrap() {
                table.add_column_justify(
                    s(column, "header").unwrap(),
                    justify(s(column, "justify").unwrap_or("left")),
                );
                if let Some(style) = s(column, "style").filter(|st| !st.is_empty()) {
                    table.column_style(Style::parse(style).map_err(|e| format!("{e:?}"))?);
                }
                if column["no_wrap"].as_bool().unwrap_or(false) {
                    table.column_no_wrap();
                }
            }
            for row in node["rows"].as_array().unwrap() {
                let cells: Vec<&str> = row
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|c| c.as_str().unwrap())
                    .collect();
                table.add_row(&cells);
            }
            Ok(Box::new(table))
        }
        "panel" => {
            let pad = node["padding"].as_array();
            let (v, h) = pad.map_or((0, 1), |p| {
                (
                    p[0].as_u64().unwrap() as usize,
                    p[1].as_u64().unwrap() as usize,
                )
            });
            let mut panel = Panel::new(build(&node["body"])?)
                .box_set(box_set(s(node, "box").unwrap()))
                .expand(node["expand"].as_bool().unwrap_or(true))
                .padding((v, h, v, h));
            if let Some(title) = s(node, "title") {
                panel = panel.title(title);
            }
            if let Some(subtitle) = s(node, "subtitle") {
                panel = panel.subtitle(subtitle);
            }
            if let Some(style) = s(node, "border_style").filter(|st| !st.is_empty()) {
                panel = panel.border_style(style);
            }
            Ok(Box::new(panel))
        }
        "tree" => {
            let root = &node["root"];
            let mut tree = Tree::new(s(root, "label").unwrap());
            if let Some(guide) = s(node, "guide_style").filter(|st| !st.is_empty()) {
                tree = tree.guide_style(guide);
            }
            add_children(&mut tree, root["children"].as_array().unwrap());
            Ok(Box::new(tree))
        }
        "progress" => {
            let width = node["bar_width"].as_u64().map(|w| w as usize);
            let mut progress = Progress::new().columns(vec![
                ProgressColumn::TextFormat(TextColumn::new("{task.description}")),
                ProgressColumn::BarWith(BarColumn::new().bar_width(width)),
                ProgressColumn::TaskProgress { show_speed: false },
                ProgressColumn::MofN,
            ]);
            for task in node["tasks"].as_array().unwrap() {
                progress.add_task(
                    s(task, "description").unwrap(),
                    task["total"].as_f64().unwrap(),
                    task["completed"].as_f64().unwrap(),
                );
            }
            Ok(Box::new(progress))
        }
        "error" => {
            let mut body = Text::new("");
            body.append(s(node, "message").unwrap(), Some("bold".into()));
            let causes = node["causes"].as_array().map_or(&[][..], |c| c.as_slice());
            if !causes.is_empty() {
                body.append("\n\n", None);
                body.append("Caused by:", Some("dim".into()));
                for (i, cause) in causes.iter().enumerate() {
                    body.append(&format!("\n    {i}: {}", cause.as_str().unwrap()), None);
                }
            }
            if let Some(hint) = s(node, "hint").filter(|h| !h.is_empty()) {
                body.append("\n\n", None);
                body.append(&format!("hint: {hint}"), Some("cyan".into()));
            }
            Ok(Box::new(
                Panel::new(Box::new(body))
                    .title("Error")
                    .border_style("red"),
            ))
        }
        other => Err(format!("unknown renderable {other}")),
    }
}

fn color_system(name: &str) -> Option<ColorSystem> {
    match name {
        "truecolor" => Some(ColorSystem::Truecolor),
        "256" => Some(ColorSystem::EightBit),
        "standard" => Some(ColorSystem::Standard),
        "none" => None,
        other => panic!("unknown color system {other}"),
    }
}

fn main() {
    let mut args = std::env::args().skip(1);
    let cases = PathBuf::from(args.next().expect("cases.jsonl"));
    let outdir = PathBuf::from(args.next().expect("outdir"));
    fs::create_dir_all(&outdir).unwrap();
    for line in fs::read_to_string(&cases).unwrap().lines() {
        if line.trim().is_empty() {
            continue;
        }
        let case: Value = serde_json::from_str(line).unwrap();
        let id = s(&case, "id").unwrap();
        let console = Console::builder()
            .width(case["width"].as_u64().unwrap() as usize)
            .height(24)
            .force_terminal(true)
            .color_system(color_system(s(&case, "color_system").unwrap()))
            .legacy_windows(false)
            .emoji(false)
            .highlight(false)
            .no_color(false)
            .build();
        match build(&case["renderable"]) {
            Ok(renderable) => {
                fs::write(
                    outdir.join(format!("{id}.ansi")),
                    console.render_export(renderable.as_ref()),
                )
                .unwrap();
            }
            Err(reason) => {
                eprintln!("{id}: unsupported: {reason}");
                fs::write(outdir.join(format!("{id}.unsupported")), "").unwrap();
            }
        }
    }
}
