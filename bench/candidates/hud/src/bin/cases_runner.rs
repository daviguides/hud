// cases-runner <cases.jsonl> <outdir>
// One <id>.ansi per case hud implements; an empty <id>.unsupported for the rest. hud claims the
// features listed in support.json: every milestone adds to it, and an unclaimed feature is
// unsupported, never a pass.
use std::fs;
use std::path::PathBuf;

use hud::{ColorSystem, Console, Style, Text};
use serde_json::{Value, json};

const CLAIMED: &[&str] = &["style", "markup"];

fn color_system(name: &str) -> ColorSystem {
    match name {
        "truecolor" => ColorSystem::TrueColor,
        "256" => ColorSystem::EightBit,
        "standard" => ColorSystem::Standard,
        "none" => ColorSystem::None,
        other => panic!("unknown color system {other}"),
    }
}

fn render(case: &Value) -> Option<String> {
    let node = &case["renderable"];
    if node["t"].as_str()? != "text" {
        return None;
    }
    let system = color_system(case["color_system"].as_str()?);
    let console = Console::builder()
        .width(case["width"].as_u64()? as u16)
        .height(24)
        .color_system(system)
        .attributes(system != ColorSystem::None)
        .build();
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
