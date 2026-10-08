//! Glue: inline markup, word wrap and ANSI-aware width. None of the three crates provides these.
use crate::style::{Depth, St, paint, parse_style};
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

#[derive(Clone, Copy)]
pub struct Cell {
    pub ch: char,
    pub st: St,
}

pub fn cw(c: char) -> usize {
    c.width().unwrap_or(0)
}

pub fn cells_width(cs: &[Cell]) -> usize {
    cs.iter().map(|c| cw(c.ch)).sum()
}

pub fn strip_ansi(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut it = s.chars().peekable();
    while let Some(c) = it.next() {
        if c == '\x1b' && it.peek() == Some(&'[') {
            it.next();
            for n in it.by_ref() {
                if ('@'..='~').contains(&n) {
                    break;
                }
            }
        } else {
            out.push(c);
        }
    }
    out
}

pub fn vis_width(s: &str) -> usize {
    strip_ansi(s).width()
}

pub fn plain_cells(text: &str, st: St) -> Vec<Cell> {
    text.chars().map(|ch| Cell { ch, st }).collect()
}

pub fn parse_markup(src: &str) -> Vec<Cell> {
    let chars: Vec<char> = src.chars().collect();
    let mut out = Vec::new();
    let mut stack: Vec<(String, St)> = Vec::new();
    let cur = |stack: &Vec<(String, St)>| stack.iter().fold(St::default(), |a, (_, s)| a.over(*s));
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        if c == '\\' && chars.get(i + 1) == Some(&'[') {
            out.push(Cell {
                ch: '[',
                st: cur(&stack),
            });
            i += 2;
            continue;
        }
        if c == '[' {
            let end = chars[i + 1..].iter().position(|&x| x == ']' || x == '[');
            if let Some(e) = end.filter(|&e| chars[i + 1 + e] == ']') {
                let tag: String = chars[i + 1..i + 1 + e].iter().collect();
                let first = tag.chars().next();
                if first.is_some_and(|f| f.is_ascii_lowercase() || "#/@".contains(f)) {
                    if let Some(name) = tag.strip_prefix('/') {
                        match stack
                            .iter()
                            .rposition(|(n, _)| name.is_empty() || n == name)
                        {
                            Some(p) => {
                                stack.remove(p);
                            }
                            None => {}
                        }
                    } else {
                        stack.push((tag.clone(), parse_style(&tag)));
                    }
                    i += e + 2;
                    continue;
                }
            }
        }
        out.push(Cell {
            ch: c,
            st: cur(&stack),
        });
        i += 1;
    }
    out
}

fn expand_tabs(line: &[Cell], tab: usize) -> Vec<Cell> {
    let mut out: Vec<Cell> = Vec::new();
    let mut col = 0;
    for c in line {
        if c.ch == '\t' {
            let pad = tab - col % tab;
            out.extend(std::iter::repeat_n(Cell { ch: ' ', st: c.st }, pad));
            col += pad;
        } else {
            col += cw(c.ch);
            out.push(*c);
        }
    }
    out
}

fn words(line: &[Cell]) -> Vec<(usize, usize)> {
    let mut out = Vec::new();
    let mut i = 0;
    while i < line.len() {
        let start = i;
        while i < line.len() && line[i].ch.is_whitespace() {
            i += 1;
        }
        while i < line.len() && !line[i].ch.is_whitespace() {
            i += 1;
        }
        while i < line.len() && line[i].ch.is_whitespace() {
            i += 1;
        }
        out.push((start, i));
    }
    out
}

fn divide_line(line: &[Cell], width: usize) -> Vec<usize> {
    let mut breaks = Vec::new();
    let mut offset = 0;
    for (start, end) in words(line) {
        let word = &line[start..end];
        let trimmed = word
            .iter()
            .rposition(|c| !c.ch.is_whitespace())
            .map_or(0, |p| p + 1);
        let word_len = cells_width(&word[..trimmed]);
        if width as i64 - offset as i64 >= word_len as i64 {
            offset += cells_width(word);
        } else if word_len > width {
            let mut chunks: Vec<(usize, usize)> = Vec::new();
            let (mut s, mut acc) = (start, 0);
            for (k, c) in word.iter().enumerate() {
                let w = cw(c.ch);
                if acc + w > width && start + k > s {
                    chunks.push((s, start + k));
                    s = start + k;
                    acc = 0;
                }
                acc += w;
            }
            chunks.push((s, end));
            for (n, (cs, ce)) in chunks.iter().enumerate() {
                if *cs > 0 {
                    breaks.push(*cs);
                }
                if n + 1 == chunks.len() {
                    offset = cells_width(&line[*cs..*ce]);
                }
            }
        } else if offset > 0 && start > 0 {
            breaks.push(start);
            offset = cells_width(word);
        }
    }
    breaks
}

pub fn wrap(cells: &[Cell], width: usize) -> Vec<Vec<Cell>> {
    let mut lines = Vec::new();
    for raw in cells.split(|c| c.ch == '\n') {
        let line = expand_tabs(raw, 8);
        let mut cuts = divide_line(&line, width);
        cuts.dedup();
        let mut prev = 0;
        cuts.push(line.len());
        for cut in cuts {
            let mut piece = line[prev..cut].to_vec();
            if piece.len() > width {
                let ws = piece
                    .iter()
                    .rev()
                    .take_while(|c| c.ch.is_whitespace())
                    .count();
                piece.truncate(piece.len() - ws.min(piece.len() - width));
            }
            lines.push(piece);
            prev = cut;
        }
    }
    lines
}

pub fn spans_to_ansi(line: &[Cell], d: Depth) -> String {
    let mut out = String::new();
    let mut i = 0;
    while i < line.len() {
        let st = line[i].st;
        let mut j = i;
        let mut text = String::new();
        while j < line.len() && line[j].st == st {
            text.push(line[j].ch);
            j += 1;
        }
        out.push_str(&paint(&text, &st, d));
        i = j;
    }
    out
}

pub fn natural_width(cells: &[Cell]) -> usize {
    cells
        .split(|c| c.ch == '\n')
        .map(|l| cells_width(&expand_tabs(l, 8)))
        .max()
        .unwrap_or(0)
}

pub fn render_text(cells: &[Cell], width: usize, d: Depth) -> Vec<String> {
    wrap(cells, width)
        .iter()
        .map(|l| spans_to_ansi(l, d))
        .collect()
}
