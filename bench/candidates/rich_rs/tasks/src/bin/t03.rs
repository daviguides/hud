use rich_rs::{
    BarColumn, LiveOptions, MofNCompleteColumn, Progress, ProgressColumn, TaskProgressColumn,
    TextColumn,
};
use std::time::Duration;

fn main() {
    let columns: Vec<Box<dyn ProgressColumn>> = vec![
        Box::new(TextColumn::new("{task.description}")),
        Box::new(BarColumn::new().with_bar_width(Some(30))),
        Box::new(TaskProgressColumn::new(false)),
        Box::new(MofNCompleteColumn::new()),
    ];
    let mut progress = Progress::new(columns, LiveOptions::default(), false, false);
    progress.start().unwrap();
    let steps = [("fetch index", 3), ("download crates", 120), ("verify", 12)];
    let ids: Vec<_> = steps
        .iter()
        .map(|(name, total)| progress.add_task(name, true, Some(*total as f64), 0.0, true))
        .collect();
    for step in 0..120 {
        for (id, (_, total)) in ids.iter().zip(steps.iter()) {
            if step < *total {
                progress.advance(*id, 1.0);
            }
        }
        std::thread::sleep(Duration::from_millis(5));
    }
    progress.stop().unwrap();
}
