#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Differential tests against Python Rich: vectors rendered by the pinned Rich
//! (`bench/scripts/gen_oracle_vectors.py`) must come out byte for byte the same.

use std::fs;
use std::path::PathBuf;

use std::sync::{Arc, Mutex};

use hud::{
    Align, BarColumn, Body, BoxStyle, ColorSystem, Column, Console, ErrorReport, Justify,
    MofNCompleteColumn, Overflow, Pad, Panel, Progress, ProgressBuilder, SpinnerColumn, Style,
    Table, TaskProgressColumn, Text, TextColumn, TimeElapsedColumn, TimeRemainingColumn, Tree,
};
use serde_json::Value;

fn fixture(name: &str) -> Vec<Value> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name);
    fs::read_to_string(path)
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect()
}

fn system(name: &str) -> ColorSystem {
    match name {
        "truecolor" => ColorSystem::TrueColor,
        "256" => ColorSystem::EightBit,
        "standard" => ColorSystem::Standard,
        _ => ColorSystem::None,
    }
}

fn console(width: u16, system: ColorSystem) -> Console {
    Console::builder()
        .width(width)
        .height(24)
        .color_system(system)
        .attributes(system != ColorSystem::None)
        .build()
}

#[test]
fn style_vectors_match_rich() {
    let mut bad = Vec::new();
    let rows = fixture("style.jsonl");
    for row in &rows {
        let input = row["input"].as_str().unwrap();
        let parsed = Style::parse(input);
        if !row["ok"].as_bool().unwrap() {
            if parsed.is_ok() {
                bad.push(format!("{input:?}: Rich rejects it, hud accepts"));
            }
            continue;
        }
        let Ok(style) = parsed else {
            bad.push(format!(
                "{input:?}: Rich accepts it, hud rejects: {}",
                parsed.unwrap_err()
            ));
            continue;
        };
        for name in ["standard", "256", "truecolor"] {
            let text = Text::styled("X", style.clone()).end("");
            let got = console(80, system(name)).render_to_string(&text);
            let want = row["ansi"][name].as_str().unwrap();
            if got != want {
                bad.push(format!("{input:?} at {name}: want {want:?}, got {got:?}"));
            }
        }
    }
    assert!(
        bad.is_empty(),
        "{} of {} style vectors differ, first:\n{}",
        bad.len(),
        rows.len(),
        bad.iter().take(15).cloned().collect::<Vec<_>>().join("\n")
    );
}

/// Rich writes a random id into every hyperlink; hud writes none (D-020).
fn strip_link_ids(text: &str) -> String {
    let mut out = String::new();
    let mut rest = text;
    while let Some(at) = rest.find("\x1b]8;id=") {
        out.push_str(&rest[..at]);
        out.push_str("\x1b]8;");
        let after = &rest[at + "\x1b]8;id=".len()..];
        let skip = after.find(';').unwrap_or(0);
        rest = &after[skip..];
    }
    out.push_str(rest);
    out
}

/// `(id, input, message)` of every vector whose bytes differ from Rich.
fn markup_mismatches(name: &str) -> (usize, Vec<(String, String, String)>) {
    let rows = fixture(name);
    let mut bad = Vec::new();
    for row in &rows {
        let id = row["id"].as_str().unwrap();
        let text = if row["kind"] == "markup" {
            Text::from_markup(row["markup"].as_str().unwrap()).map_err(|e| e.to_string())
        } else {
            let style = Style::parse(row["style"].as_str().unwrap()).unwrap();
            Ok(Text::styled(row["plain"].as_str().unwrap(), style))
        };
        let text = match (text, row.get("error")) {
            (Err(_), Some(_)) => continue,
            (Err(e), None) => {
                bad.push((
                    id.to_string(),
                    String::new(),
                    format!("{id}: Rich renders it, hud errors: {e}"),
                ));
                continue;
            }
            (Ok(_), Some(error)) => {
                bad.push((
                    id.to_string(),
                    String::new(),
                    format!(
                        "{id}: Rich raises {error} for {:?}, hud accepts",
                        row["markup"]
                    ),
                ));
                continue;
            }
            (Ok(text), None) => text,
        };
        let justify = match row["justify"].as_str().unwrap() {
            "left" => Justify::Left,
            "center" => Justify::Center,
            "right" => Justify::Right,
            "full" => Justify::Full,
            _ => Justify::Default,
        };
        let overflow = match row["overflow"].as_str().unwrap() {
            "crop" => Overflow::Crop,
            "ellipsis" => Overflow::Ellipsis,
            "ignore" => Overflow::Ignore,
            _ => Overflow::Fold,
        };
        let text = text
            .justify(justify)
            .overflow(overflow)
            .no_wrap(row["no_wrap"].as_bool().unwrap())
            .tab_size(row["tab_size"].as_u64().unwrap() as usize)
            .end(row["end"].as_str().unwrap());
        let width = row["width"].as_u64().unwrap() as u16;
        let got =
            console(width, system(row["color_system"].as_str().unwrap())).render_to_string(&text);
        let want = strip_link_ids(row["ansi"].as_str().unwrap());
        if got != want {
            let source = if row["kind"] == "markup" {
                &row["markup"]
            } else {
                &row["plain"]
            };
            bad.push((
                id.to_string(),
                source.as_str().unwrap_or_default().to_string(),
                format!(
                    "{id} w={width} {} {} {}: input {source:?} style {:?}\n   want {want:?}\n   got  {got:?}",
                    row["color_system"], row["justify"], row["overflow"], row.get("style")
                ),
            ));
        }
    }
    (rows.len(), bad)
}

#[test]
fn ascii_markup_and_styled_text_match_rich() {
    let (total, bad) = markup_mismatches("markup_ascii.jsonl");
    assert!(
        bad.is_empty(),
        "{} of {total} ascii vectors differ, first:\n{}",
        bad.len(),
        bad.iter()
            .take(12)
            .map(|b| b.2.clone())
            .collect::<Vec<_>>()
            .join("\n")
    );
}

/// A difference with Rich that is on purpose: D-003 (hud never splits a flag or another cluster
/// when folding) and D-024 (hud measures trailing whitespace in cells where Rich counts
/// characters, so a line with a combining mark keeps one more cell). Both need a flag or a
/// combining mark in the input; a difference without one is a bug.
fn explained(input: &str) -> bool {
    input
        .chars()
        .any(|c| c == '\u{301}' || ('\u{1F1E6}'..='\u{1F1FF}').contains(&c))
}

#[test]
fn unicode_markup_differs_from_rich_only_with_flags_or_combining_marks() {
    let (total, bad) = markup_mismatches("markup_unicode.jsonl");
    let unexplained: Vec<_> = bad
        .iter()
        .filter(|(_, input, _)| !explained(input))
        .collect();
    assert!(
        unexplained.is_empty(),
        "{} of {total} unicode vectors differ, {} without a flag or combining mark, first:\n{}",
        bad.len(),
        unexplained.len(),
        unexplained
            .iter()
            .take(8)
            .map(|b| b.2.clone())
            .collect::<Vec<_>>()
            .join("\n")
    );
    assert!(
        bad.len() * 25 < total,
        "{} of {total} is more than the expected few",
        bad.len()
    );
}

fn table_from(row: &Value) -> Result<Table, String> {
    let boxes = [
        ("rounded", BoxStyle::Rounded),
        ("ascii", BoxStyle::Ascii),
        ("simple", BoxStyle::Simple),
        ("heavy", BoxStyle::Heavy),
        ("double", BoxStyle::Double),
        ("minimal", BoxStyle::Minimal),
        ("square", BoxStyle::Square),
        ("heavy_head", BoxStyle::HeavyHead),
    ];
    let name = row["box"].as_str().unwrap();
    let mut table = Table::new()
        .box_style(boxes.iter().find(|(n, _)| *n == name).unwrap().1)
        .show_lines(row["show_lines"].as_bool().unwrap());
    if let Some(title) = row["title"].as_str() {
        table = table.title(title);
    }
    if let Some(caption) = row["caption"].as_str() {
        table = table.caption(caption);
    }
    if let Some(style) = row["header_style"].as_str() {
        table = table.header_style(Style::parse(style).map_err(|e| e.to_string())?);
    }
    for column in row["columns"].as_array().unwrap() {
        let justify = match column["justify"].as_str().unwrap() {
            "center" => Justify::Center,
            "right" => Justify::Right,
            _ => Justify::Left,
        };
        let overflow = match column["overflow"].as_str().unwrap_or("ellipsis") {
            "crop" => Overflow::Crop,
            "fold" => Overflow::Fold,
            "ignore" => Overflow::Ignore,
            _ => Overflow::Ellipsis,
        };
        let mut built = Column::new(column["header"].as_str().unwrap())
            .justify(justify)
            .overflow(overflow)
            .no_wrap(column["no_wrap"].as_bool().unwrap());
        let style = column["style"].as_str().unwrap();
        if !style.is_empty() {
            built = built.style(Style::parse(style).map_err(|e| e.to_string())?);
        }
        if let Some(style) = column["header_style"].as_str() {
            built = built.header_style(Style::parse(style).map_err(|e| e.to_string())?);
        }
        for (key, apply) in [
            ("width", Column::width as fn(Column, usize) -> Column),
            ("min_width", Column::min_width),
            ("max_width", Column::max_width),
        ] {
            if let Some(value) = column[key].as_u64() {
                built = apply(built, value as usize);
            }
        }
        table.add_column(built);
    }
    for cells in row["rows"].as_array().unwrap() {
        table.add_row(
            cells
                .as_array()
                .unwrap()
                .iter()
                .map(|c| c.as_str().unwrap()),
        );
    }
    Ok(table)
}

/// `(id, input, message)` of every table vector whose bytes differ from Rich.
fn table_mismatches(name: &str) -> (usize, Vec<(String, String, String)>) {
    let rows = fixture(name);
    let mut bad = Vec::new();
    for row in &rows {
        let id = row["id"].as_str().unwrap();
        let mut spec = row.clone();
        spec.as_object_mut().unwrap().remove("ansi");
        let input = spec.to_string();
        let table = table_from(row);
        match (table, row.get("error")) {
            (_, Some(_)) => continue,
            (Err(e), None) => {
                bad.push((
                    id.to_string(),
                    input,
                    format!("{id}: Rich renders it, hud errors: {e}"),
                ));
                continue;
            }
            (Ok(table), None) => {
                let width = row["width"].as_u64().unwrap() as u16;
                let got = console(width, system(row["color_system"].as_str().unwrap()))
                    .render_to_string(&table);
                let want = strip_link_ids(row["ansi"].as_str().unwrap());
                if got != want {
                    if std::env::var("TABLE_DEBUG").is_ok_and(|v| v == id) {
                        let first = want
                            .lines()
                            .zip(got.lines())
                            .position(|(a, b)| a != b)
                            .unwrap_or(0);
                        eprintln!(
                            "DEBUG {id}\nINPUT {input}\nline {first}\nWANT {:?}\nGOT  {:?}\nFULL-WANT\n{want}\nFULL-GOT\n{got}",
                            want.lines().nth(first),
                            got.lines().nth(first)
                        );
                    }
                    bad.push((
                        id.to_string(),
                        input,
                        format!(
                            "{id} w={width} {}\n   want {want:?}\n   got  {got:?}",
                            row["color_system"]
                        ),
                    ));
                }
            }
        }
    }
    (rows.len(), bad)
}

#[test]
fn ascii_tables_match_rich() {
    let (total, bad) = table_mismatches("table_ascii.jsonl");
    assert!(
        bad.is_empty(),
        "{} of {total} ascii table vectors differ, first:\n{}",
        bad.len(),
        bad.iter()
            .take(6)
            .map(|b| b.2.clone())
            .collect::<Vec<_>>()
            .join("\n")
    );
}

/// D-024 seen from a table: Rich compares the characters of a line with the width where hud
/// compares cells. With a wide character (two cells, one character) Rich keeps a trailing space
/// it should cut, so a word that exactly fits gets an ellipsis; the line Rich prints has `…` where
/// hud's does not.
fn explained_by_wide_characters(input: &str, want: &str, got: &str) -> bool {
    let wide = input
        .chars()
        .any(|c| hud::cell_width(c.encode_utf8(&mut [0; 4])) > 1);
    wide && want
        .lines()
        .zip(got.lines())
        .filter(|(a, b)| a != b)
        .all(|(a, b)| ellipses(&without_escapes(a)) > ellipses(&without_escapes(b)))
}

fn ellipses(line: &str) -> usize {
    line.matches('…').count()
}

fn without_escapes(line: &str) -> String {
    let mut out = String::new();
    let mut in_escape = false;
    for c in line.chars() {
        match (in_escape, c) {
            (false, '\u{1b}') => in_escape = true,
            (true, 'm') => in_escape = false,
            (false, _) => out.push(c),
            (true, _) => {}
        }
    }
    out
}

#[test]
fn unicode_tables_differ_from_rich_only_where_d003_or_d024_say_so() {
    let rows = fixture("table_unicode.jsonl");
    let (total, bad) = table_mismatches("table_unicode.jsonl");
    let mut unexplained = Vec::new();
    let (mut marks, mut wide) = (0, 0);
    for (id, input, message) in &bad {
        if explained(input) {
            marks += 1;
            continue;
        }
        let row = rows.iter().find(|r| r["id"] == id.as_str()).unwrap();
        let table = table_from(row).unwrap();
        let got = console(
            row["width"].as_u64().unwrap() as u16,
            system(row["color_system"].as_str().unwrap()),
        )
        .render_to_string(&table);
        if explained_by_wide_characters(input, row["ansi"].as_str().unwrap(), &got) {
            wide += 1;
        } else {
            unexplained.push(message.clone());
        }
    }
    eprintln!(
        "{total} unicode table vectors: {} differ ({marks} flag or combining mark, {wide} wide character)",
        bad.len()
    );
    assert!(
        unexplained.is_empty(),
        "{} unexplained, first:\n{}",
        unexplained.len(),
        unexplained
            .iter()
            .take(6)
            .cloned()
            .collect::<Vec<_>>()
            .join("\n")
    );
}

fn align(name: &str) -> Align {
    match name {
        "left" => Align::Left,
        "right" => Align::Right,
        _ => Align::Center,
    }
}

fn style_of(text: &str) -> Result<Style, String> {
    Style::parse(text).map_err(|e| e.to_string())
}

fn body_from(node: &Value) -> Result<Body, String> {
    Ok(match node["t"].as_str().unwrap() {
        "text" => Body::from(node["markup"].as_str().unwrap()),
        "table" => Body::from(table_from(node)?),
        "tree" => Body::from(tree_from(node)?),
        _ => Body::from(panel_from(node)?),
    })
}

fn panel_from(node: &Value) -> Result<Panel, String> {
    let boxes = [
        ("rounded", BoxStyle::Rounded),
        ("ascii", BoxStyle::Ascii),
        ("simple", BoxStyle::Simple),
        ("heavy", BoxStyle::Heavy),
        ("double", BoxStyle::Double),
        ("minimal", BoxStyle::Minimal),
        ("square", BoxStyle::Square),
        ("heavy_head", BoxStyle::HeavyHead),
    ];
    let name = node["box"].as_str().unwrap();
    let pad: Vec<usize> = node["padding"]
        .as_array()
        .unwrap()
        .iter()
        .map(|n| n.as_u64().unwrap() as usize)
        .collect();
    let padding = match pad.as_slice() {
        [all] => Pad::from(*all),
        [vertical, horizontal] => Pad::from((*vertical, *horizontal)),
        [top, right, bottom, left] => Pad::from((*top, *right, *bottom, *left)),
        other => panic!("padding of {} numbers", other.len()),
    };
    let mut panel = Panel::new(body_from(&node["body"])?)
        .box_style(boxes.iter().find(|(n, _)| *n == name).unwrap().1)
        .expand(node["expand"].as_bool().unwrap())
        .padding(padding)
        .title_align(align(node["title_align"].as_str().unwrap()))
        .subtitle_align(align(node["subtitle_align"].as_str().unwrap()));
    if let Some(title) = node["title"].as_str() {
        panel = panel.title(title);
    }
    if let Some(subtitle) = node["subtitle"].as_str() {
        panel = panel.subtitle(subtitle);
    }
    let border = node["border_style"].as_str().unwrap();
    if !border.is_empty() {
        panel = panel.border_style(style_of(border)?);
    }
    Ok(panel)
}

fn tree_node(node: &Value) -> Result<Tree, String> {
    let mut tree = Tree::new(node["label"].as_str().unwrap());
    if let Some(style) = node["guide_style"].as_str() {
        tree = tree.guide_style(style_of(style)?);
    }
    for child in node["children"].as_array().unwrap() {
        tree = tree.child(tree_node(child)?);
    }
    Ok(tree)
}

fn tree_from(node: &Value) -> Result<Tree, String> {
    let mut root = tree_node(&node["root"])?;
    let guide = node["guide_style"].as_str().unwrap();
    if !guide.is_empty() {
        root = root.guide_style(style_of(guide)?);
    }
    Ok(root)
}

fn error_from(node: &Value) -> ErrorReport {
    let mut report = ErrorReport::new(node["message"].as_str().unwrap());
    for cause in node["causes"].as_array().unwrap() {
        report = report.cause(cause.as_str().unwrap());
    }
    if let Some(hint) = node["hint"].as_str() {
        report = report.hint(hint);
    }
    report
}

/// `(id, input, message)` of every panel, tree or error vector whose bytes differ from Rich.
fn widget_mismatches(name: &str) -> (usize, Vec<(String, String, String)>) {
    let rows = fixture(name);
    let mut bad = Vec::new();
    for row in &rows {
        let id = row["id"].as_str().unwrap();
        if row.get("error").is_some() {
            continue;
        }
        let input = row["node"].to_string();
        let node = &row["node"];
        let width = row["width"].as_u64().unwrap() as u16;
        let console = console(width, system(row["color_system"].as_str().unwrap()));
        let rendered = match row["kind"].as_str().unwrap() {
            "panel" => panel_from(node).map(|p| console.render_to_string(&p)),
            "error" => Ok(console.render_to_string(&error_from(node))),
            _ => tree_from(node).map(|t| console.render_to_string(&t)),
        };
        match rendered {
            Err(e) => bad.push((
                id.to_string(),
                input,
                format!("{id}: Rich renders it, hud errors: {e}"),
            )),
            Ok(got) => {
                let want = strip_link_ids(row["ansi"].as_str().unwrap());
                if got != want {
                    bad.push((
                        id.to_string(),
                        input,
                        format!(
                            "{id} w={width} {}\n   want {want:?}\n   got  {got:?}",
                            row["color_system"]
                        ),
                    ));
                }
            }
        }
    }
    (rows.len(), bad)
}

fn assert_all_match(name: &str) {
    let (total, bad) = widget_mismatches(name);
    assert!(
        bad.is_empty(),
        "{} of {total} {name} vectors differ, first:\n{}",
        bad.len(),
        bad.iter()
            .take(6)
            .map(|b| b.2.clone())
            .collect::<Vec<_>>()
            .join("\n")
    );
}

#[test]
fn ascii_panels_match_rich() {
    assert_all_match("panel_ascii.jsonl");
}

#[test]
fn ascii_trees_match_rich() {
    assert_all_match("tree_ascii.jsonl");
}

#[test]
fn ascii_error_reports_match_rich() {
    assert_all_match("error_ascii.jsonl");
}

fn assert_only_explained_differences(name: &str) {
    let rows = fixture(name);
    let (total, bad) = widget_mismatches(name);
    let (mut marks, mut wide) = (0, 0);
    let mut unexplained = Vec::new();
    for (id, input, message) in &bad {
        if explained(input) {
            marks += 1;
            continue;
        }
        let row = rows.iter().find(|r| r["id"] == id.as_str()).unwrap();
        let console = console(
            row["width"].as_u64().unwrap() as u16,
            system(row["color_system"].as_str().unwrap()),
        );
        let got = match row["kind"].as_str().unwrap() {
            "panel" => console.render_to_string(&panel_from(&row["node"]).unwrap()),
            "error" => console.render_to_string(&error_from(&row["node"])),
            _ => console.render_to_string(&tree_from(&row["node"]).unwrap()),
        };
        if explained_by_wide_characters(input, row["ansi"].as_str().unwrap(), &got) {
            wide += 1;
        } else {
            let want = row["ansi"].as_str().unwrap();
            let first = want.lines().zip(got.lines()).find(|(a, b)| a != b);
            unexplained.push(format!("{message}\n   first differing line: {first:?}"));
        }
    }
    eprintln!(
        "{total} {name} vectors: {} differ ({marks} flag or combining mark, {wide} wide character)",
        bad.len()
    );
    assert!(
        unexplained.is_empty(),
        "{} unexplained, first:\n{}",
        unexplained.len(),
        unexplained
            .iter()
            .take(6)
            .cloned()
            .collect::<Vec<_>>()
            .join("\n")
    );
}

#[test]
fn unicode_panels_differ_from_rich_only_where_the_deviations_say_so() {
    assert_only_explained_differences("panel_unicode.jsonl");
}

#[test]
fn unicode_trees_differ_from_rich_only_where_the_deviations_say_so() {
    assert_only_explained_differences("tree_unicode.jsonl");
}

#[test]
fn unicode_error_reports_differ_from_rich_only_where_the_deviations_say_so() {
    assert_only_explained_differences("error_unicode.jsonl");
}

fn progress_builder(row: &Value, clock: &Arc<Mutex<f64>>) -> ProgressBuilder {
    let clock = Arc::clone(clock);
    let mut builder = Progress::builder()
        .clock(move || *clock.lock().unwrap())
        .disable(true);
    for column in row["columns"].as_array().unwrap() {
        builder = match column["t"].as_str().unwrap() {
            "text" => builder.column(TextColumn::new(column["template"].as_str().unwrap())),
            "bar" => builder.column(match column["bar_width"].as_u64() {
                Some(width) => BarColumn::new().bar_width(width as usize),
                None => BarColumn::new().full_width(),
            }),
            "percent" => builder.column(TaskProgressColumn::new()),
            "mofn" => builder
                .column(MofNCompleteColumn::new().separator(column["separator"].as_str().unwrap())),
            "elapsed" => builder.column(TimeElapsedColumn::new()),
            "remaining" => builder.column(
                TimeRemainingColumn::new()
                    .compact(column["compact"].as_bool().unwrap())
                    .elapsed_when_finished(column["elapsed_when_finished"].as_bool().unwrap()),
            ),
            _ => builder.column(
                SpinnerColumn::new()
                    .speed(column["speed"].as_f64().unwrap())
                    .finished_text(column["finished_text"].as_str().unwrap()),
            ),
        };
    }
    builder
}

/// Replays the events of a vector on a fake clock and renders twice, like the generator: the
/// first render starts the spinner, the second is the one compared.
fn progress_output(row: &Value) -> String {
    let clock = Arc::new(Mutex::new(0.0_f64));
    let progress = progress_builder(row, &clock).build();
    let mut tasks = Vec::new();
    for event in row["events"].as_array().unwrap() {
        let event = event.as_array().unwrap();
        *clock.lock().unwrap() = event[0].as_f64().unwrap();
        let number = event[2].as_u64().unwrap() as usize;
        match event[1].as_str().unwrap() {
            "add" => {
                let task = &row["tasks"][number];
                tasks.push(progress.add_task(
                    task["description"].as_str().unwrap(),
                    task["total"].as_u64().unwrap(),
                ));
            }
            "advance" => progress.advance(&tasks[number], event[3].as_u64().unwrap()),
            "update" => {
                progress
                    .update(&tasks[number])
                    .completed(event[3].as_u64().unwrap());
            }
            _ => {
                progress
                    .update(&tasks[number])
                    .total(event[3].as_u64().unwrap());
            }
        }
    }
    let width = row["width"].as_u64().unwrap() as u16;
    let console = console(width, system(row["color_system"].as_str().unwrap()));
    *clock.lock().unwrap() = row["first"].as_f64().unwrap();
    let _ = console.render_to_string(&progress);
    *clock.lock().unwrap() = row["now"].as_f64().unwrap();
    console.render_to_string(&progress)
}

/// `(id, input, message)` of every progress vector whose bytes differ from Rich.
fn progress_mismatches(name: &str) -> (usize, Vec<(String, String, String)>) {
    let rows = fixture(name);
    let mut bad = Vec::new();
    for row in &rows {
        if row.get("error").is_some() {
            continue;
        }
        let id = row["id"].as_str().unwrap();
        let got = progress_output(row);
        let want = row["ansi"].as_str().unwrap();
        if got != want {
            let input = row["tasks"].to_string();
            bad.push((
                id.to_string(),
                input,
                format!(
                    "{id} w={} {} columns {}\n   events {}\n   want {want:?}\n   got  {got:?}",
                    row["width"], row["color_system"], row["columns"], row["events"]
                ),
            ));
        }
    }
    (rows.len(), bad)
}

#[test]
fn ascii_progress_matches_rich() {
    let (total, bad) = progress_mismatches("progress_ascii.jsonl");
    assert!(
        bad.is_empty(),
        "{} of {total} ascii progress vectors differ, first:\n{}",
        bad.len(),
        bad.iter()
            .take(6)
            .map(|b| b.2.clone())
            .collect::<Vec<_>>()
            .join("\n")
    );
}

#[test]
fn unicode_progress_differs_from_rich_only_where_d003_or_d024_say_so() {
    let (total, bad) = progress_mismatches("progress_unicode.jsonl");
    let unexplained: Vec<_> = bad
        .iter()
        .filter(|(_, input, message)| {
            !explained(input) && !explained_by_wide_characters(input, "", message)
        })
        .collect();
    eprintln!(
        "{total} unicode progress vectors: {} differ, {} unexplained",
        bad.len(),
        unexplained.len()
    );
    assert!(
        unexplained.is_empty(),
        "{} unexplained unicode progress vectors, first:\n{}",
        unexplained.len(),
        unexplained
            .iter()
            .take(6)
            .map(|b| b.2.clone())
            .collect::<Vec<_>>()
            .join("\n")
    );
}
