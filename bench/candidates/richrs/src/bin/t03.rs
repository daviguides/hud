use richrs::console::ColorSystem;
use richrs::prelude::*;

fn main() -> Result<()> {
    let mut console = Console::new();
    if !console.is_terminal() || std::env::var_os("NO_COLOR").is_some() {
        console.set_color_system(ColorSystem::None);
    }

    let mut progress = Progress::new().bar(ProgressBar::new().width(30));
    let tasks = [
        (progress.add_task("fetch index    ", Some(3), true), 3),
        (progress.add_task("download crates", Some(120), true), 120),
        (progress.add_task("verify         ", Some(12), true), 12),
    ];
    for (task, steps) in tasks {
        for _ in 0..steps {
            progress.advance(task, 1)?;
        }
    }

    console.write_segments(&progress.render(console.width()))
}
