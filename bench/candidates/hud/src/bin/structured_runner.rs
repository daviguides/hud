// structured-runner <structured.jsonl> <outdir>
// Corpus 4 (cases/structured.jsonl): one nested renderable per case, written in the three formats of
// one console (`<id>.json`, `<id>.plain`, `<id>.rich`), plus `report.json` with the checks that need
// the library itself: the same value rendered as JSON on five widths fifty times each must give the
// same bytes (criterion S2), and a document read by `Node::from_json` must write back identically
// (the round trip of S3). Everything else (schema validity, canonical form, the Python oracle) is
// checked by `scripts/structured.py` from the files.
use std::fs;
use std::path::PathBuf;

use hud::{
    Body, ColorSystem, Column, Columns, Console, ErrorReport, Format, Group, Justify, Layout, Node,
    Padding, Panel, Progress, Table, Tree,
};
use serde_json::{Value, json};

fn color_system(name: &str) -> ColorSystem {
    match name {
        "truecolor" => ColorSystem::TrueColor,
        "256" => ColorSystem::EightBit,
        "standard" => ColorSystem::Standard,
        _ => ColorSystem::None,
    }
}

fn justify(name: Option<&str>) -> Option<Justify> {
    Some(match name? {
        "left" => Justify::Left,
        "center" => Justify::Center,
        "right" => Justify::Right,
        "full" => Justify::Full,
        _ => return None,
    })
}

fn text_of(value: &Value) -> Option<&str> {
    value.as_str()
}

fn table(node: &Value) -> Option<Table> {
    let mut built = Table::new();
    if let Some(title) = text_of(&node["title"]) {
        built = built.title(title);
    }
    if let Some(caption) = text_of(&node["caption"]) {
        built = built.caption(caption);
    }
    for column in node["columns"].as_array()? {
        let mut made = Column::new(column["header"].as_str()?);
        if let Some(j) = justify(column["justify"].as_str()) {
            made = made.justify(j);
        }
        built = built.column(made);
    }
    for row in node["rows"].as_array()? {
        built = built.row(
            row.as_array()?
                .iter()
                .map(|cell| cell.as_str().map(str::to_string))
                .collect::<Option<Vec<_>>>()?,
        );
    }
    Some(built)
}

fn tree(node: &Value) -> Option<Tree> {
    let mut built = Tree::new(node["label"].as_str()?);
    for child in node["children"].as_array()? {
        built = built.child(tree(child)?);
    }
    Some(built)
}

fn layout(node: &Value) -> Option<Layout> {
    let children = node["children"].as_array()?;
    let mut built = if !children.is_empty() {
        let kids = children.iter().map(layout).collect::<Option<Vec<_>>>()?;
        match node["split"].as_str()? {
            "row" => Layout::row(kids),
            _ => Layout::column(kids),
        }
    } else if node["body"].is_null() {
        Layout::empty()
    } else {
        Layout::new(build(&node["body"])?)
    };
    if let Some(name) = node["name"].as_str() {
        built = built.name(name);
    }
    if let Some(size) = node["size"].as_u64() {
        built = built.size(size as usize);
    }
    built = built.ratio(node["ratio"].as_u64()? as usize);
    built = built.visible(node["visible"].as_bool()?);
    Some(built)
}

fn build(node: &Value) -> Option<Body> {
    Some(match node["t"].as_str()? {
        "text" => Body::from(node["markup"].as_str()?),
        "table" => Body::from(table(node)?),
        "panel" => {
            let mut panel = Panel::new(build(&node["body"])?);
            if let Some(title) = node["title"].as_str() {
                panel = panel.title(title);
            }
            if let Some(subtitle) = node["subtitle"].as_str() {
                panel = panel.subtitle(subtitle);
            }
            Body::from(panel)
        }
        "tree" => Body::from(tree(&node["root"])?),
        "progress" => {
            let progress = Progress::builder().disable(true).build();
            for task in node["tasks"].as_array()? {
                let made = progress.add_task(task["description"].as_str()?, task["total"].as_u64()?);
                made.set_completed(task["completed"].as_u64()?);
                made.set_visible(task["visible"].as_bool()?);
            }
            Body::new(progress)
        }
        "error" => {
            let mut report = ErrorReport::new(node["message"].as_str()?);
            for cause in node["causes"].as_array()? {
                report = report.cause(cause.as_str()?);
            }
            if let Some(hint) = node["hint"].as_str() {
                report = report.hint(hint);
            }
            Body::new(report)
        }
        "columns" => {
            let items = node["items"]
                .as_array()?
                .iter()
                .map(build)
                .collect::<Option<Vec<_>>>()?;
            let mut columns = Columns::new(items);
            if let Some(title) = node["title"].as_str() {
                columns = columns.title(title);
            }
            Body::from(columns)
        }
        "layout" => Body::from(layout(&node["root"])?),
        "group" => {
            let mut group = Group::new();
            for item in node["items"].as_array()? {
                group = group.push(build(item)?);
            }
            Body::from(group)
        }
        "padding" => {
            let pad = node["pad"].as_array()?;
            let at = |i: usize| pad.get(i).and_then(Value::as_u64).map(|n| n as usize);
            Body::from(Padding::new(
                build(&node["body"])?,
                (at(0)?, at(1)?, at(2)?, at(3)?),
            ))
        }
        _ => return None,
    })
}

fn console_for(case: &Value, width: u16) -> Option<Console> {
    let system = color_system(case["color_system"].as_str()?);
    Some(
        Console::builder()
            .width(width)
            .height(case["height"].as_u64().unwrap_or(24) as u16)
            .color_system(system)
            .attributes(system != ColorSystem::None)
            .build(),
    )
}

const WIDTHS: [u16; 5] = [20, 40, 80, 100, 200];
const REPEATS: usize = 50;

fn main() {
    let mut args = std::env::args().skip(1);
    let cases = PathBuf::from(args.next().expect("structured.jsonl"));
    let outdir = PathBuf::from(args.next().expect("outdir"));
    fs::create_dir_all(&outdir).unwrap();
    let (mut total, mut renders, mut mismatches, mut roundtrip) = (0usize, 0usize, 0usize, 0usize);
    let mut unsupported = Vec::new();
    for line in fs::read_to_string(cases)
        .unwrap()
        .lines()
        .filter(|l| !l.trim().is_empty())
    {
        let case: Value = serde_json::from_str(line).unwrap();
        let id = case["id"].as_str().unwrap();
        total += 1;
        let (Some(body), Some(console)) = (
            build(&case["renderable"]),
            console_for(&case, case["width"].as_u64().unwrap_or(80) as u16),
        ) else {
            unsupported.push(id.to_string());
            continue;
        };
        let json = console.render_as(&body, Format::Json);
        for width in WIDTHS {
            let other = console_for(&case, width).unwrap();
            for _ in 0..REPEATS {
                renders += 1;
                if other.render_as(&body, Format::Json) != json {
                    mismatches += 1;
                }
            }
        }
        match Node::from_json(&json) {
            Ok(node) if node.to_json() == json => {}
            _ => roundtrip += 1,
        }
        fs::write(outdir.join(format!("{id}.json")), &json).unwrap();
        fs::write(
            outdir.join(format!("{id}.plain")),
            console.render_as(&body, Format::Plain),
        )
        .unwrap();
        fs::write(
            outdir.join(format!("{id}.rich")),
            console.render_as(&body, Format::Rich),
        )
        .unwrap();
    }
    fs::write(
        outdir.join("report.json"),
        json!({
            "cases": total,
            "unsupported": unsupported,
            "json_renders": renders,
            "determinism_mismatches": mismatches,
            "roundtrip_failures": roundtrip,
            "widths": WIDTHS,
            "repeats": REPEATS,
        })
        .to_string(),
    )
    .unwrap();
}
