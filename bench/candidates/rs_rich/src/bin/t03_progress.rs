use std::time::Duration;

use rich::{BarColumn, Console, Progress, ProgressColumn, TextColumn};

fn main() {
    let steps = [("fetch index", 3), ("download crates", 120), ("verify", 12)];
    let progress = Progress::new().columns(vec![
        ProgressColumn::TextFormat(TextColumn::new("{task.description}")),
        ProgressColumn::BarWith(BarColumn::new().bar_width(Some(30))),
        ProgressColumn::Percentage,
        ProgressColumn::MofN,
    ]);
    let live = progress.start(Console::new(), std::io::stdout(), 10.0);
    let ids: Vec<_> = steps
        .iter()
        .map(|(name, total)| live.add_task(*name, *total as f64, 0.0))
        .collect();
    for step in 0..120 {
        for (id, (_, total)) in ids.iter().zip(steps) {
            if step < total {
                live.advance(*id, 1.0);
            }
        }
        std::thread::sleep(Duration::from_millis(5));
    }
    live.stop();
}
