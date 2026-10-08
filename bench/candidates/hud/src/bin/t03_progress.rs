use hud::{BarColumn, MofNCompleteColumn, Progress, TaskProgressColumn, TextColumn};

fn main() {
    let progress = Progress::builder()
        .column(TextColumn::new("{task.description}"))
        .column(BarColumn::new().bar_width(30))
        .column(TaskProgressColumn::new())
        .column(MofNCompleteColumn::new())
        .build();
    let steps = [("fetch index", 3), ("download crates", 120), ("verify", 12)];
    let tasks: Vec<_> = steps.iter().map(|&(name, total)| progress.add_task(name, total)).collect();
    for step in 0..120 {
        for (task, &(_, total)) in tasks.iter().zip(&steps) {
            if step < total {
                progress.advance(task, 1);
            }
        }
    }
}
