use hud::Progress;

fn main() {
    let progress = Progress::new();
    let task = progress.add_task("Downloading", 100);
    task.advance(100);
}
