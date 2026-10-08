use richrs::prelude::*;

fn main() -> Result<()> {
    let mut progress = Progress::new();
    let task = progress.add_task("Downloading", Some(100), true);

    for _ in 0..100 {
        progress.advance(task, 1)?;
        std::thread::sleep(std::time::Duration::from_millis(20));
    }

    Ok(())
}
