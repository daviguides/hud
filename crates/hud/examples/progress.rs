//! Two tasks that advance together and finish. On a terminal the display redraws in place;
//! when the output is piped it prints once.
//!
//! ```bash
//! cargo run -p hud --example progress
//! ```

use std::thread::sleep;
use std::time::Duration;

use hud::Progress;

fn main() {
    let progress = Progress::new();
    let compile = progress.add_task("compile", 60);
    let test = progress.add_task("test", 30);
    for step in 0..60 {
        progress.advance(&compile, 1);
        if step % 2 == 0 {
            progress.advance(&test, 1);
        }
        sleep(Duration::from_millis(25));
    }
    progress.finish();
}
