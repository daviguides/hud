//! A panel with a title and a subtitle around wrapped text.
//!
//! ```bash
//! cargo run -p hud --example panel
//! ```

use hud::{Console, Panel};

fn main() {
    let text = "The cache was rebuilt because three source files changed. \
                Wide characters such as 世界 and emoji like 🚀 keep the border aligned.";
    let panel = Panel::new(text).title("Cache").subtitle("rebuilt");
    Console::stdout().print(&panel);
}
