// status-runner <status.jsonl> <outdir>
// Corpus 5: one <id>.ansi per Spinner and Status case, written like live_layout_runner.rs writes
// corpus 3. The clock is part of the case: every frame is drawn at the stated time.
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use hud::{ColorSystem, Console, Spinner, Status, Style};
use serde_json::{Value, json};

pub(crate) const CLAIMED: &[&str] = &["spinner", "status"];

fn color_system(name: &str) -> Option<ColorSystem> {
    Some(match name {
        "truecolor" => ColorSystem::TrueColor,
        "256" => ColorSystem::EightBit,
        "standard" => ColorSystem::Standard,
        "none" => ColorSystem::None,
        _ => return None,
    })
}

fn console_for(case: &Value) -> Option<Console> {
    let system = color_system(case["color_system"].as_str()?)?;
    Some(
        Console::builder()
            .width(case["width"].as_u64()? as u16)
            .height(case["height"].as_u64().unwrap_or(24) as u16)
            .color_system(system)
            .attributes(system != ColorSystem::None)
            .build(),
    )
}

#[derive(Clone, Default)]
struct Capture(Arc<Mutex<Vec<u8>>>);

impl Write for Capture {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.0.lock().unwrap().extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

type Clock = Arc<Mutex<f64>>;

fn clock_fn(clock: &Clock) -> impl Fn() -> f64 + Send + Sync + 'static {
    let clock = Arc::clone(clock);
    move || *clock.lock().unwrap()
}

fn set(clock: &Clock, at: f64) {
    *clock.lock().unwrap() = at;
}

fn style(definition: &Value) -> Option<Style> {
    definition.as_str().and_then(|s| Style::parse(s).ok())
}

/// Every animation printed at the stated clock readings, with the updates of the case.
fn render_spinner(case: &Value) -> Option<String> {
    let node = &case["renderable"];
    let clock = Clock::default();
    let mut spinner = Spinner::try_new(node["name"].as_str()?)
        .ok()?
        .speed(node["speed"].as_f64()?)
        .clock(clock_fn(&clock));
    if let Some(text) = node["text"].as_str() {
        spinner = spinner.text(text);
    }
    if let Some(style) = style(&node["style"]) {
        spinner = spinner.style(style);
    }
    let console = console_for(case)?;
    let mut out = String::new();
    for op in node["ops"].as_array()? {
        match op["op"].as_str()? {
            "print" => {
                set(&clock, op["at"].as_f64()?);
                out.push_str(&console.render_to_string(&spinner));
            }
            "update" => {
                let mut update = spinner.update().text(op["text"].as_str()?);
                if let Some(style) = style(&op["style"]) {
                    update = update.style(style);
                }
                update.speed(op["speed"].as_f64()?);
            }
            _ => return None,
        }
    }
    Some(out)
}

/// `start`, `refresh` and `update` at the stated times, `stop`: every byte written to the stream.
fn render_status(case: &Value) -> Option<String> {
    let node = &case["renderable"];
    let clock = Clock::default();
    let capture = Capture::default();
    let mut builder = Status::builder()
        .text(node["text"].as_str()?)
        .console(console_for(case)?)
        .spinner(node["spinner"].as_str()?)
        .speed(node["speed"].as_f64()?)
        .interactive(node["terminal"].as_bool()?)
        .auto_refresh(false)
        .clock(clock_fn(&clock))
        .writer(capture.clone());
    if let Some(style) = style(&node["spinner_style"]) {
        builder = builder.spinner_style(style);
    }
    let status = builder.build();
    for event in node["events"].as_array()? {
        set(&clock, event["at"].as_f64()?);
        match event["op"].as_str()? {
            "start" => status.start(),
            "refresh" => status.refresh(),
            "update" => {
                let mut update = status.update();
                if let Some(text) = event["text"].as_str() {
                    update = update.text(text);
                }
                if let Some(name) = event["spinner"].as_str() {
                    update = update.spinner(name);
                }
                if let Some(style) = style(&event["spinner_style"]) {
                    update = update.spinner_style(style);
                }
                if let Some(speed) = event["speed"].as_f64() {
                    update = update.speed(speed);
                }
                drop(update);
            }
            "stop" => status.stop(),
            _ => return None,
        }
    }
    let bytes = capture.0.lock().unwrap().clone();
    String::from_utf8(bytes).ok()
}

/// The bytes hud writes for one case, or `None` when the case is not one it supports.
pub(crate) fn render(case: &Value) -> Option<String> {
    match case["renderable"]["t"].as_str()? {
        "spinner" => render_spinner(case),
        "status" => render_status(case),
        _ => None,
    }
}

fn main() {
    let mut args = std::env::args().skip(1);
    let cases = PathBuf::from(args.next().expect("status.jsonl"));
    let outdir = PathBuf::from(args.next().expect("outdir"));
    fs::create_dir_all(&outdir).unwrap();
    for line in fs::read_to_string(cases)
        .unwrap()
        .lines()
        .filter(|l| !l.trim().is_empty())
    {
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
