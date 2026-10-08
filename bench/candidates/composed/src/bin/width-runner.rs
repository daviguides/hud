use hud_bench_composed::render::render;
use hud_bench_composed::style::Depth;
use serde_json::{Value, json};
use std::fs;
use std::io::Write;
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

fn fold(text: &str, w: usize) -> Vec<String> {
    let (mut lines, mut cur, mut cur_w) = (Vec::new(), String::new(), 0);
    for g in text.graphemes(true) {
        let gw = g.width();
        if !cur.is_empty() && cur_w + gw > w {
            lines.push(std::mem::take(&mut cur));
            cur_w = 0;
        }
        cur.push_str(g);
        cur_w += gw;
    }
    if !cur.is_empty() || lines.is_empty() {
        lines.push(cur);
    }
    lines
}

fn jsonl(path: &str) -> Vec<Value> {
    fs::read_to_string(path)
        .unwrap()
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| serde_json::from_str(l).unwrap())
        .collect()
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let (cases, out) = (&args[1], std::path::Path::new(&args[2]));
    fs::create_dir_all(out.join("tables")).unwrap();
    let mut width = fs::File::create(out.join("width.jsonl")).unwrap();
    let mut fold_f = fs::File::create(out.join("fold.jsonl")).unwrap();
    let mut trunc = fs::File::create(out.join("truncate.jsonl")).unwrap();
    for item in jsonl(&format!("{cases}/width_corpus.jsonl")) {
        let (id, text) = (item["id"].as_str().unwrap(), item["text"].as_str().unwrap());
        writeln!(width, "{}", json!({ "id": id, "width": text.width() })).unwrap();
        for w in 1..=40 {
            let lines = fold(text, w);
            writeln!(fold_f, "{}", json!({ "id": id, "w": w, "lines": lines })).unwrap();
            writeln!(trunc, "{}", json!({ "id": id, "w": w, "text": lines[0] })).unwrap();
        }
    }
    for case in jsonl(&format!("{cases}/table_unicode.jsonl")) {
        let lines = render(
            &case["renderable"],
            case["width"].as_u64().unwrap() as usize,
            Depth::None,
        );
        fs::write(
            out.join("tables")
                .join(format!("{}.ansi", case["id"].as_str().unwrap())),
            lines.join("\n") + "\n",
        )
        .unwrap();
    }
}
