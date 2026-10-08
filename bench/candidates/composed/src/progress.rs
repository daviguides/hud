//! Glue: progress frames through indicatif. indicatif draws to a terminal, so a frame is captured
//! with a hand-written `TermLike`, and the column widths Rich would pick are computed here.
use crate::style::{Col, Depth, downgrade};
use indicatif::{ProgressBar, ProgressDrawTarget, ProgressStyle, TermLike};
use serde_json::Value;
use std::fmt::Debug;
use std::io;
use std::sync::{Arc, Mutex};

#[derive(Debug)]
struct Capture {
    width: u16,
    buf: Arc<Mutex<String>>,
}

impl TermLike for Capture {
    fn width(&self) -> u16 {
        self.width
    }
    fn move_cursor_up(&self, _: usize) -> io::Result<()> {
        Ok(())
    }
    fn move_cursor_down(&self, _: usize) -> io::Result<()> {
        Ok(())
    }
    fn move_cursor_right(&self, _: usize) -> io::Result<()> {
        Ok(())
    }
    fn move_cursor_left(&self, _: usize) -> io::Result<()> {
        Ok(())
    }
    fn write_line(&self, s: &str) -> io::Result<()> {
        self.buf.lock().unwrap().push_str(s);
        Ok(())
    }
    fn write_str(&self, s: &str) -> io::Result<()> {
        self.buf.lock().unwrap().push_str(s);
        Ok(())
    }
    fn clear_line(&self) -> io::Result<()> {
        self.buf.lock().unwrap().clear();
        Ok(())
    }
    fn flush(&self) -> io::Result<()> {
        Ok(())
    }
}

fn spec(c: Col, d: Depth) -> String {
    match downgrade(c, d) {
        Col::Rgb(r, g, b) => format!("#{r:02x}{g:02x}{b:02x}"),
        Col::Idx(n) => n.to_string(),
        Col::Std(i) => {
            let names = [
                "black", "red", "green", "yellow", "blue", "magenta", "cyan", "white",
            ];
            if i < 8 {
                names[i as usize].to_string()
            } else {
                format!("{}.bright", names[i as usize - 8])
            }
        }
        Col::Default => String::new(),
    }
}

fn collapse(widths: &mut [usize], max: usize) {
    loop {
        let total: usize = widths.iter().sum();
        if total <= max {
            return;
        }
        let top = *widths.iter().max().unwrap();
        let second = widths
            .iter()
            .copied()
            .filter(|w| *w != top)
            .max()
            .unwrap_or(0);
        let n = widths.iter().filter(|w| **w == top).count();
        let cut = ((total - max).min(top - second) / n).max(1);
        for w in widths.iter_mut().filter(|w| **w == top) {
            *w -= cut;
        }
    }
}

pub fn render_frame(node: &Value, width: usize, d: Depth) -> Vec<String> {
    let tasks: Vec<(String, u64, u64)> = node["tasks"]
        .as_array()
        .unwrap()
        .iter()
        .map(|t| {
            (
                t["description"].as_str().unwrap().into(),
                t["total"].as_u64().unwrap(),
                t["completed"].as_u64().unwrap(),
            )
        })
        .collect();
    let bar_w = node["bar_width"].as_u64().unwrap() as usize;
    console::set_colors_enabled(d != Depth::None);
    let dw = tasks
        .iter()
        .map(|t| unicode_width::UnicodeWidthStr::width(t.0.as_str()))
        .max()
        .unwrap_or(0);
    let cell = |t: &(String, u64, u64)| {
        let tw = t.1.to_string().len();
        format!("{:>tw$}/{}", t.2, t.1)
    };
    let mofn = tasks.iter().map(|t| cell(t).len()).max().unwrap_or(0);
    let mut widths = [dw + 1, bar_w + 1, 5, mofn];
    collapse(&mut widths, width);
    let bw = widths[1] - 1;
    let finished = |t: &(String, u64, u64)| t.2 >= t.1;
    tasks
        .iter()
        .map(|t| {
            let (fill, rest) = if finished(t) {
                (spec(Col::Rgb(114, 156, 31), d), spec(Col::Idx(237), d))
            } else {
                (spec(Col::Rgb(249, 38, 114), d), spec(Col::Idx(237), d))
            };
            let tw = t.1.to_string().len();
            let pad = " ".repeat(mofn - cell(t).len());
            let tpl = format!(
                "{{prefix:<{dw}}} {{bar:{bw}.{fill}/{rest}}} {{percent:>3.magenta}}% {{pos:>{tw}.green}}/{{len:.green}}"
            );
            let buf = Arc::new(Mutex::new(String::new()));
            let target = ProgressDrawTarget::term_like(Box::new(Capture { width: width as u16, buf: buf.clone() }));
            let pb = ProgressBar::with_draw_target(Some(t.1), target)
                .with_style(ProgressStyle::with_template(&tpl).unwrap().progress_chars(if d == Depth::None { "━╸ " } else { "━╸━" }))
                .with_prefix(t.0.clone());
            pb.set_position(t.2);
            pb.abandon();
            let line = buf.lock().unwrap().trim_end().to_string() + &pad;
            line
        })
        .collect()
}
