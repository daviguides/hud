#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Differential tests against Python Rich: vectors rendered by the pinned Rich
//! (`bench/scripts/gen_oracle_vectors.py`) must come out byte for byte the same.

use std::fs;
use std::path::PathBuf;

use hud::{BoxStyle, ColorSystem, Column, Console, Justify, Overflow, Style, Table, Text};
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
        let overflow = match column["overflow"].as_str().unwrap() {
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
        .all(|(a, b)| a.contains('…') && !b.contains('…'))
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
