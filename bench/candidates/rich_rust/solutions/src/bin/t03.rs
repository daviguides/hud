use rich_rust::prelude::*;
use rich_rust::renderables::Renderable;

struct Tasks(Vec<(&'static str, u64, u64)>);

impl Renderable for Tasks {
    fn render<'a>(&'a self, _console: &Console, options: &ConsoleOptions) -> Vec<Segment<'a>> {
        let mut segments = Vec::new();
        for (name, total, done) in &self.0 {
            let mut bar = ProgressBar::with_total(*total)
                .width(30)
                .bar_style(BarStyle::Line)
                .show_brackets(false)
                .description(format!("{name:<15}"));
            bar.update(*done);
            let mut line = bar.render(options.max_width);
            line.pop();
            line.push(Segment::plain(format!(" {done}/{total}")));
            line.push(Segment::line());
            segments.extend(line);
        }
        segments
    }
}

fn main() {
    let steps = [("fetch index", 3), ("download crates", 120), ("verify", 12)];
    let live = Live::new(Console::new().shared());
    let frame = |step: u64| Tasks(steps.iter().map(|&(name, total)| (name, total, step.min(total))).collect());
    let live = live.renderable(frame(0));
    live.start(true).unwrap();
    for step in 1..=120 {
        live.update(frame(step), true);
    }
    live.stop().unwrap();
}
