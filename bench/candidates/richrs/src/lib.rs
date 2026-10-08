//! Case interpreter shared by cases-runner and width-runner: renders declarative cases through richrs' public API.
use std::fs;
use std::path::Path;

use richrs::box_chars::BoxChars;
use richrs::console::ColorSystem;
use richrs::prelude::*;
use richrs::table::Row;
use serde_json::Value;

type Res<T> = std::result::Result<T, String>;

fn s<'a>(v: &'a Value, k: &str) -> Option<&'a str> {
    v.get(k).and_then(Value::as_str).filter(|x| !x.is_empty())
}

fn e<T: std::fmt::Display>(err: T) -> String {
    err.to_string()
}

fn style(spec: Option<&str>) -> Res<Option<Style>> {
    spec.map(|t| Style::parse(t).map_err(e)).transpose()
}

fn markup(text: &str) -> Res<Text> {
    Ok(Markup::parse(text).map_err(e)?.to_text())
}

fn box_chars(name: &str) -> Res<BoxChars> {
    Ok(match name {
        "rounded" => BoxChars::ROUNDED,
        "ascii" => BoxChars::ASCII,
        "simple" => BoxChars::SIMPLE,
        "heavy" => BoxChars::HEAVY,
        "double" => BoxChars::DOUBLE,
        "minimal" => BoxChars::MINIMAL,
        "square" => BoxChars::SQUARE,
        other => return Err(format!("no `{other}` box in BoxChars")),
    })
}

fn tree_node(v: &Value) -> Res<TreeNode> {
    let mut node = TreeNode::new(markup(s(v, "label").unwrap_or(""))?);
    for child in v["children"].as_array().into_iter().flatten() {
        node = node.with_child(tree_node(child)?);
    }
    Ok(node)
}

pub fn render(console: &mut Console, r: &Value, width: usize) -> Res<()> {
    match r["t"].as_str().unwrap_or("") {
        "text" => {
            if let Some(m) = s(r, "markup") {
                console.print(m).map_err(e)
            } else {
                let st = style(s(r, "style"))?.unwrap_or_default();
                console
                    .print_styled(r["plain"].as_str().unwrap_or(""), &st)
                    .map_err(e)
            }
        }
        "table" => {
            let mut table = Table::new()
                .box_chars(Some(box_chars(r["box"].as_str().unwrap_or("square"))?))
                .show_lines(r["show_lines"].as_bool().unwrap_or(false));
            if let Some(t) = s(r, "title") {
                table = table.title(t);
            }
            if let Some(c) = s(r, "caption") {
                table = table.caption(c);
            }
            for col in r["columns"].as_array().into_iter().flatten() {
                let mut c = Column::new(s(col, "header").unwrap_or(""));
                c = c.justify(match col["justify"].as_str() {
                    Some("right") => Justify::Right,
                    Some("center") => Justify::Center,
                    _ => Justify::Left,
                });
                c = c.no_wrap(col["no_wrap"].as_bool().unwrap_or(false));
                if let Some(st) = style(s(col, "style"))? {
                    c = c.style(st);
                }
                table.add_column(c);
            }
            for row in r["rows"].as_array().into_iter().flatten() {
                let cells: Res<Vec<Text>> = row
                    .as_array()
                    .into_iter()
                    .flatten()
                    .map(|c| markup(c.as_str().unwrap_or("")))
                    .collect();
                table.add_row(Row::new(cells?));
            }
            console.write_segments(&table.render(width)).map_err(e)
        }
        "panel" => {
            let body = &r["body"];
            if body["t"] != "text" {
                return Err("Panel::new takes Into<Text>; no nested renderable body".into());
            }
            let text = match s(body, "markup") {
                Some(m) => markup(m)?,
                None => Text::from(body["plain"].as_str().unwrap_or("")),
            };
            let pad = r["padding"].as_array().cloned().unwrap_or_default();
            let v = pad.first().and_then(Value::as_u64).unwrap_or(0) as usize;
            let h = pad.get(1).and_then(Value::as_u64).unwrap_or(1) as usize;
            let mut panel = Panel::new(text)
                .box_chars(box_chars(r["box"].as_str().unwrap_or("rounded"))?)
                .expand(r["expand"].as_bool().unwrap_or(true))
                .padding(h, v, h, v);
            if let Some(t) = s(r, "title") {
                panel = panel.title(t);
            }
            if let Some(t) = s(r, "subtitle") {
                panel = panel.subtitle(t);
            }
            if let Some(st) = style(s(r, "border_style"))? {
                panel = panel.border_style(st);
            }
            console.write_segments(&panel.render(width)).map_err(e)
        }
        "tree" => {
            let root = &r["root"];
            let mut tree = Tree::new(markup(s(root, "label").unwrap_or(""))?);
            for child in root["children"].as_array().into_iter().flatten() {
                tree.add(tree_node(child)?);
            }
            if let Some(st) = style(s(r, "guide_style"))? {
                tree = tree.guide_style(st);
            }
            console.write_segments(&tree.render()).map_err(e)
        }
        "progress" => {
            let bar = ProgressBar::new().width(r["bar_width"].as_u64().unwrap_or(30) as usize);
            let mut progress = Progress::new().bar(bar);
            for t in r["tasks"].as_array().into_iter().flatten() {
                let id = progress.add_task(
                    t["description"].as_str().unwrap_or(""),
                    t["total"].as_u64(),
                    true,
                );
                progress
                    .update(id, t["completed"].as_u64(), None, None, None)
                    .map_err(e)?;
            }
            console.write_segments(&progress.render(width)).map_err(e)
        }
        "error" => {
            let mut body = Text::new();
            body.append(r["message"].as_str().unwrap_or(""), style(Some("bold"))?);
            let causes = r["causes"].as_array().cloned().unwrap_or_default();
            if !causes.is_empty() {
                body.append("\n\n", None);
                body.append("Caused by:", style(Some("dim"))?);
                for (i, c) in causes.iter().enumerate() {
                    body.append(&format!("\n    {i}: {}", c.as_str().unwrap_or("")), None);
                }
            }
            if let Some(h) = s(r, "hint") {
                body.append("\n\n", None);
                body.append(&format!("hint: {h}"), style(Some("cyan"))?);
            }
            let panel = Panel::new(body)
                .title("Error")
                .border_style(style(Some("red"))?.unwrap_or_default());
            console.write_segments(&panel.render(width)).map_err(e)
        }
        other => Err(format!("unknown renderable {other}")),
    }
}

pub fn run_cases(cases: &Path, out: &Path) {
    fs::create_dir_all(out).expect("outdir");
    for line in fs::read_to_string(cases).expect("cases").lines() {

        let case: Value = serde_json::from_str(line).expect("json");
        let id = case["id"].as_str().expect("id");
        let width = case["width"].as_u64().unwrap_or(80) as usize;
        let mut console = Console::new();
        console.set_force_terminal(true);
        console.set_width(width);
        console.set_height(24);
        console.set_color_system(match case["color_system"].as_str() {
            Some("truecolor") => ColorSystem::TrueColor,
            Some("256") => ColorSystem::EightBit,
            Some("standard") => ColorSystem::Standard,
            _ => ColorSystem::None,
        });
        console.options_mut().emoji = false;
        console.options_mut().highlight = false;
        console.begin_capture();
        let result = render(&mut console, &case["renderable"], width);
        let captured = console.end_capture();
        match result {
            Ok(()) => fs::write(out.join(format!("{id}.ansi")), captured).expect("write"),
            Err(reason) => fs::write(out.join(format!("{id}.unsupported")), reason).expect("write"),
        }
    }
}
