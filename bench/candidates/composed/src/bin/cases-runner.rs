use hud_bench_composed::render::render;
use hud_bench_composed::style::Depth;
use std::fs;
use std::panic::catch_unwind;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let out = std::path::Path::new(&args[2]);
    fs::create_dir_all(out).unwrap();
    let mut features = std::collections::BTreeSet::new();
    for line in fs::read_to_string(&args[1])
        .unwrap()
        .lines()
        .filter(|l| !l.trim().is_empty())
    {
        let case: serde_json::Value = serde_json::from_str(line).unwrap();
        let id = case["id"].as_str().unwrap().to_string();
        features.insert(case["feature"].as_str().unwrap().to_string());
        let depth = match case["color_system"].as_str().unwrap() {
            "truecolor" => Depth::True,
            "256" => Depth::Idx,
            "standard" => Depth::Std,
            _ => Depth::None,
        };
        let width = case["width"].as_u64().unwrap() as usize;
        match catch_unwind(|| render(&case["renderable"], width, depth)) {
            Ok(lines) => {
                fs::write(out.join(format!("{id}.ansi")), lines.join("\n") + "\n").unwrap()
            }
            Err(_) => fs::write(out.join(format!("{id}.panic")), "").unwrap(),
        }
    }
    let list: Vec<_> = features.into_iter().collect();
    fs::write(
        out.join("support.json"),
        serde_json::json!({ "features": list }).to_string(),
    )
    .unwrap();
}
